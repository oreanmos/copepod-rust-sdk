use serde::{Deserialize, Serialize};

/// First-touch attribution recorded with a lead. Every field is optional; the
/// server limits each to 200 bytes without control characters.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeadAcquisition {
    /// `utm_source` query value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utm_source: Option<String>,
    /// `utm_medium` query value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utm_medium: Option<String>,
    /// `utm_campaign` query value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utm_campaign: Option<String>,
    /// Free-form referral code (`ref` on the wire).
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub referral: Option<String>,
    /// Host of the referring page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referrer_host: Option<String>,
    /// Landing path; must start with `/` (the server drops any query or fragment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landing_path: Option<String>,
}

/// Request body for [`CopepodClient::create_lead`](crate::CopepodClient::create_lead).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateLeadRequest {
    /// Audience list key, `^[a-z][a-z0-9_]{0,39}$`.
    pub list_key: String,
    /// The visitor's email address (at most 320 bytes).
    pub email: String,
    /// The consent sentence the visitor saw, verbatim (1-500 characters).
    pub consent_text: String,
    /// ISO 3166-1 alpha-2 country code, upper case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// First-touch attribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition: Option<LeadAcquisition>,
    /// The visitor's IP address, used by the server for rate limiting and
    /// Turnstile only and never stored. Pass it from SSR code; without it the
    /// server rate-limits the whole API key connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    /// Cloudflare Turnstile token; required only when the app enables Turnstile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turnstile_token: Option<String>,
}

/// Response of a lead sign-up: always `accepted: true` for a well-formed request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateLeadResponse {
    /// Always `true`; the response never reveals whether the address was known.
    pub accepted: bool,
}

/// Marketing opt-in sent with app-user registration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketingConsent {
    /// The consent sentence the user saw, verbatim (1-500 characters).
    pub consent_text: String,
}

/// Banner severity, ordered by the server `critical`, `warning`, `info`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementSeverity {
    /// Neutral notice.
    Info,
    /// Needs attention.
    Warning,
    /// Urgent.
    Critical,
}

/// An announcement currently inside its display window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveAnnouncement {
    /// Announcement id.
    pub id: String,
    /// Plain text, 1-280 characters. Render as text, never as markup.
    pub message: String,
    /// Display severity.
    pub severity: AnnouncementSeverity,
    /// Optional absolute `https://` link.
    #[serde(default)]
    pub link_url: Option<String>,
    /// Start of the window (RFC 3339, UTC).
    pub starts_at: String,
    /// End of the window, if bounded.
    #[serde(default)]
    pub ends_at: Option<String>,
}

/// Envelope of the active announcements endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ActiveAnnouncementsResponse {
    pub announcements: Vec<ActiveAnnouncement>,
}
