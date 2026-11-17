//! Analytics ingestion (launch-g2): `track_events` posts app events to
//! `api/platform/orgs/{org}/apps/{app}/analytics/events` with the client's API
//! key. See `docs/analytics.md`.

use reqwest::Method;
use serde::Serialize;

use super::email::{validate_app_scope, validate_resource_id};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::analytics::{event_names, is_app_event_name, is_copepod_event_name};
use crate::models::{AnalyticsEvent, TrackEventsResponse};

/// A batch carries 1–100 events (the server's `batch_too_large` limit).
const MAX_BATCH_EVENTS: usize = 100;

/// At most 10 properties per event.
const MAX_PROPS_KEYS: usize = 10;

/// Request body: `{"events": [...]}`.
#[derive(Serialize)]
struct TrackEventsBody<'a> {
    events: &'a [AnalyticsEvent],
}

fn invalid(message: impl Into<String>) -> CopepodError {
    CopepodError::InvalidArgument(message.into())
}

/// Mirror the server's per-event name rule: one of the four app names, or
/// `custom:` plus `^[a-z][a-z0-9_]{0,47}$`. A Copepod-only name is refused
/// here so nothing is counted twice.
fn validate_event_name(index: usize, name: &str) -> Result<()> {
    if is_app_event_name(name) {
        return Ok(());
    }
    if is_copepod_event_name(name) {
        return Err(invalid(format!(
            "events[{index}]: `{name}` is emitted by Copepod itself and cannot be sent by an app; \
             send one of {} or `custom:<name>`",
            event_names::APP_EVENT_NAMES.join(", ")
        )));
    }
    Err(invalid(format!(
        "events[{index}]: `{name}` is not an app event name; use one of {} or `custom:<name>` \
         with a name matching ^[a-z][a-z0-9_]{{0,47}}$",
        event_names::APP_EVENT_NAMES.join(", ")
    )))
}

/// At most 10 properties, each a string, number or bool (the server's
/// `invalid_props` rule; the key pattern and byte caps stay server-side).
fn validate_props(index: usize, props: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    if props.len() > MAX_PROPS_KEYS {
        return Err(invalid(format!(
            "events[{index}].props carries {} keys; at most {MAX_PROPS_KEYS} are allowed",
            props.len()
        )));
    }
    for (key, value) in props {
        if !(value.is_string() || value.is_number() || value.is_boolean()) {
            return Err(invalid(format!(
                "events[{index}].props.{key} must be a string, number or bool"
            )));
        }
    }
    Ok(())
}

/// Validate a whole batch before any network call: 1–100 events, an
/// app-sendable name, a valid `user_id` and at most 10 scalar props each.
fn validate_events(events: &[AnalyticsEvent]) -> Result<()> {
    if events.is_empty() || events.len() > MAX_BATCH_EVENTS {
        return Err(invalid(format!(
            "a batch carries 1–{MAX_BATCH_EVENTS} events, not {}",
            events.len()
        )));
    }
    for (index, event) in events.iter().enumerate() {
        validate_event_name(index, &event.name)?;
        if let Some(user_id) = &event.user_id {
            validate_resource_id(&format!("events[{index}].user_id"), user_id)?;
        }
        if let Some(props) = &event.props {
            validate_props(index, props)?;
        }
    }
    Ok(())
}

impl CopepodClient {
    /// Send a batch of analytics events via
    /// `POST api/platform/orgs/{org}/apps/{app}/analytics/events`.
    ///
    /// Requires an API key with scope `analytics:write`; no Bearer token is
    /// sent. The batch holds 1–100 events; batch size, event names, props and
    /// ids are validated before any network call and rejected with
    /// [`CopepodError::InvalidArgument`]. The server answers `202` with
    /// per-event acceptances, filtered counts and rejections; non-2xx
    /// responses arrive as [`CopepodError::Api`] with the server's code
    /// preserved (for example `batch_too_large`).
    ///
    /// Send `ip` and `user_agent` from the app's server, never from a browser,
    /// and keep `occurred_at` within the server's 48-hour lateness window. See
    /// `docs/analytics.md`.
    pub async fn track_events(
        &self,
        org_id: &str,
        app_id: &str,
        events: &[AnalyticsEvent],
    ) -> Result<TrackEventsResponse> {
        validate_app_scope(org_id, app_id)?;
        validate_events(events)?;
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/analytics/events");
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(&TrackEventsBody { events })
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(name: &str) -> AnalyticsEvent {
        AnalyticsEvent {
            name: name.to_string(),
            occurred_at: chrono::Utc::now(),
            user_id: None,
            path: None,
            referrer: None,
            ip: None,
            user_agent: None,
            acquisition: None,
            props: None,
        }
    }

    /// The name rule accepts the four app names and valid custom names, and
    /// refuses Copepod-only names with a message naming Copepod.
    #[test]
    fn event_name_rule_matches_the_server() {
        for name in event_names::APP_EVENT_NAMES {
            assert!(validate_event_name(0, name).is_ok(), "{name}");
        }
        assert!(validate_event_name(0, "custom:owner_check").is_ok());
        assert_eq!(
            event_names::custom("owner_check").unwrap(),
            "custom:owner_check"
        );
        assert!(event_names::custom("Owner").is_err());
        assert!(event_names::custom("").is_err());
        assert!(event_names::custom("has-dash").is_err());
        assert!(validate_event_name(0, "custom:UPPER").is_err());
        assert!(validate_event_name(0, "page-view").is_err());
        let reserved = validate_event_name(2, "signup_completed").unwrap_err();
        assert!(reserved.to_string().contains("Copepod"), "{reserved}");
    }

    /// Batch size and props are refused offline, with the event index named.
    #[test]
    fn batch_and_props_limits_are_enforced() {
        assert!(validate_events(&[]).is_err());
        let too_many: Vec<AnalyticsEvent> = (0..=MAX_BATCH_EVENTS)
            .map(|_| event(event_names::PAGE_VIEW))
            .collect();
        assert!(validate_events(&too_many).is_err());

        let mut props = serde_json::Map::new();
        for i in 0..=MAX_PROPS_KEYS {
            props.insert(format!("p{i}"), serde_json::json!(i));
        }
        let mut with_props = event(event_names::PAGE_VIEW);
        with_props.props = Some(props);
        let message = validate_events(&[with_props]).unwrap_err().to_string();
        assert!(message.contains("events[0].props"), "{message}");

        let mut nested = event(event_names::PAGE_VIEW);
        nested.props = Some(
            serde_json::json!({"plan": {"tier": "pro"}})
                .as_object()
                .unwrap()
                .clone(),
        );
        assert!(validate_events(&[nested]).is_err());

        let mut bad_user = event(event_names::PAGE_VIEW);
        bad_user.user_id = Some("../other".into());
        assert!(validate_events(&[bad_user]).is_err());
    }
}
