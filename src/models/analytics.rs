//! Analytics ingestion models (launch-g2): the batch an app sends to
//! `POST api/platform/orgs/{org}/apps/{app}/analytics/events` and the ingest
//! response. Timestamps are RFC 3339 UTC on the wire.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::LeadAcquisition;

/// The canonical event names an app may send. Every other app-visible name
/// must be `custom:<name>`; the names Copepod emits itself (for example
/// `signup_completed`) are refused. See `docs/analytics.md`.
pub mod event_names {
    use super::is_valid_custom_name;
    use crate::error::{CopepodError, Result};

    /// A page was viewed.
    pub const PAGE_VIEW: &str = "page_view";
    /// A visitor reached the signup form for the first time in a session.
    pub const SIGNUP_STARTED: &str = "signup_started";
    /// A signed-up user reached the app's activation moment.
    pub const ACTIVATED: &str = "activated";
    /// A checkout was started.
    pub const CHECKOUT_STARTED: &str = "checkout_started";

    /// The four names an app may send without the `custom:` prefix.
    pub const APP_EVENT_NAMES: [&str; 4] = [PAGE_VIEW, SIGNUP_STARTED, ACTIVATED, CHECKOUT_STARTED];

    /// The names only Copepod emits (registration, leads, the billing watcher).
    /// An app that sends one is refused with `reserved_event`.
    pub const COPEPOD_EVENT_NAMES: [&str; 9] = [
        "signup_completed",
        "trial_started",
        "lead_created",
        "lead_confirmed",
        "subscription_started",
        "subscription_renewed",
        "subscription_cancelled",
        "subscription_expired",
        "payment_failed",
    ];

    /// Build a `custom:<name>` event name.
    ///
    /// `name` must match `^[a-z][a-z0-9_]{0,47}$`, the server's rule; anything
    /// else returns [`CopepodError::InvalidArgument`] without a request.
    pub fn custom(name: &str) -> Result<String> {
        if !is_valid_custom_name(name) {
            return Err(CopepodError::InvalidArgument(format!(
                "custom event name `{name}` must match ^[a-z][a-z0-9_]{{0,47}}$"
            )));
        }
        Ok(format!("custom:{name}"))
    }
}

/// Whether `name` is one of the four [`event_names::APP_EVENT_NAMES`] or
/// `custom:` plus a valid custom name.
pub(crate) fn is_app_event_name(name: &str) -> bool {
    event_names::APP_EVENT_NAMES.contains(&name)
        || name
            .strip_prefix("custom:")
            .is_some_and(is_valid_custom_name)
}

/// Whether `name` is one Copepod emits itself, which apps must not send.
pub(crate) fn is_copepod_event_name(name: &str) -> bool {
    event_names::COPEPOD_EVENT_NAMES.contains(&name)
}

/// `^[a-z][a-z0-9_]{0,47}$`, byte-wise so non-ASCII fails.
fn is_valid_custom_name(name: &str) -> bool {
    (1..=48).contains(&name.len())
        && name.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// One analytics event in a [`CopepodClient::track_events`](crate::CopepodClient::track_events)
/// batch. Only `name` and `occurred_at` are required; every optional field is
/// omitted from the JSON when `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Event name: one of the [`event_names`] constants or `custom:<name>`.
    pub name: String,
    /// When the event happened; serialised as RFC 3339 UTC.
    pub occurred_at: DateTime<Utc>,
    /// The app user's id (`[A-Za-z0-9_-]`, 1–128 bytes); required for
    /// `activated`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Page path; must start with `/` and is at most 512 bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Referring page URL; only its host is stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// The visitor's IP address as the server sees it. Never sent from a
    /// browser and never stored; with `user_agent` it feeds the anonymous
    /// daily visitor hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// The visitor's user agent. Never stored; feeds the visitor hash and the
    /// bot filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// First-touch attribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition: Option<LeadAcquisition>,
    /// Up to 10 scalar properties (string, number or bool values).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Response of a `track_events` batch: `202` even when single events inside it
/// were rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEventsResponse {
    /// Events stored.
    pub accepted: u32,
    /// Events dropped as bots or excluded (test) users.
    pub filtered: u32,
    /// Events refused, with the index in the request batch and the server's
    /// rejection code.
    #[serde(default)]
    pub rejected: Vec<RejectedEvent>,
}

/// One refused event inside a [`TrackEventsResponse`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedEvent {
    /// Index of the event in the request batch.
    pub index: u32,
    /// The server's rejection code (for example `reserved_event`,
    /// `invalid_user_id`, `invalid_props`).
    pub code: String,
}
