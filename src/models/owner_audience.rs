//! Owner-scoped audiences (publishing P8): one newsletter list per app user.

use serde::{Deserialize, Serialize};

/// Lifecycle of an owner audience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OwnerAudienceStatus {
    Active,
    Suspended,
    /// A status added by a newer server.
    #[serde(other)]
    Unknown,
}

/// Where a subscriber is in double opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OwnerSubscriberStatus {
    Pending,
    Confirmed,
    Unsubscribed,
    /// A status added by a newer server.
    #[serde(other)]
    Unknown,
}

/// Lifecycle of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OwnerIssueStatus {
    Draft,
    Tested,
    Sending,
    Sent,
    /// Cancelled by a suspension or disable; it can no longer be tested or sent.
    Cancelled,
    /// A status added by a newer server.
    #[serde(other)]
    Unknown,
}

/// `PUT …/owner-audiences/{owner_user_id}` request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutOwnerAudience {
    pub display_name: String,
    pub site_url: String,
    /// Off unless sent; the server defaults an omitted field to `false`.
    #[serde(default)]
    pub reply_to_owner: bool,
    pub consent_text: String,
    pub enabled: bool,
}

/// API response: the audience plus limits and live counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerAudience {
    pub owner_user_id: String,
    pub display_name: String,
    pub site_url: String,
    pub reply_to_owner: bool,
    pub consent_text: String,
    pub enabled: bool,
    pub status: OwnerAudienceStatus,
    pub limits: OwnerAudienceLimits,
    pub counts: OwnerAudienceCounts,
    pub issues_this_month: u64,
    #[serde(default)]
    pub disabled_at: Option<String>,
}

/// Plan limits. `u64::MAX` means unlimited (see [`OwnerAudienceLimits::UNLIMITED`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerAudienceLimits {
    pub subscribers_max: u64,
    pub issues_per_month: u64,
}

impl OwnerAudienceLimits {
    /// The value the server sends for an unlimited plan.
    pub const UNLIMITED: u64 = u64::MAX;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnerAudienceCounts {
    pub pending: u64,
    pub confirmed: u64,
    pub unsubscribed: u64,
}

/// `POST …/owner-audiences/{owner_user_id}/subscribers` request body (the reader sign-up).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOwnerSubscriber {
    pub email: String,
    pub consent_text: String,
    pub client_ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

/// The 202 answer of the sign-up, test and send endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerAccepted {
    pub accepted: bool,
}

/// One subscriber row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerSubscriber {
    pub id: String,
    pub email: String,
    pub status: OwnerSubscriberStatus,
    pub created_at: String,
    #[serde(default)]
    pub confirmed_at: Option<String>,
    #[serde(default)]
    pub unsubscribed_at: Option<String>,
}

/// A page of items from an owner list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPage<T> {
    pub items: Vec<T>,
    pub total: u64,
}

/// Query of the subscriber list endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriberQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OwnerSubscriberStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// `POST …/issues` request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOwnerIssue {
    pub subject: String,
    pub body_html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_source: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerIssue {
    pub id: String,
    pub status: OwnerIssueStatus,
    pub subject: String,
    pub content_hash: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counts: Option<OwnerIssueCounts>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnerIssueCounts {
    pub sent: u64,
    pub skipped_unsubscribed: u64,
    pub skipped_unconfirmed: u64,
    pub failed: u64,
}

/// `POST …/issues/{issue_id}/test` request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIssueRequest {
    pub to: String,
}
