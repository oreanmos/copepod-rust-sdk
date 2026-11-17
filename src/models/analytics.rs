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
    /// A page was viewed.
    pub const PAGE_VIEW: &str = "page_view";
    /// A visitor reached the signup form for the first time in a session.
    pub const SIGNUP_STARTED: &str = "signup_started";
    /// A signed-up user reached the app's activation moment.
    pub const ACTIVATED: &str = "activated";
    /// A checkout was started.
    pub const CHECKOUT_STARTED: &str = "checkout_started";

    /// Build a `custom:<name>` event name.
    pub fn custom(name: &str) -> String {
        // The pattern check lands with the implementation (launch-g2 S8).
        let _ = name;
        unimplemented!("launch-g2 S8")
    }
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
