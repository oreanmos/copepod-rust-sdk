//! Trust and safety: content reports, the operator review queue, the actions
//! feed and the abuse ledger (AI3 C9-C11).

use serde::{Deserialize, Serialize};

use super::trust::{trust_enum, RestrictionScope};

trust_enum! {
    /// What a reporter says is wrong.
    ReportCategory {
        IllegalContent => "illegal_content",
        ChildSafety => "child_safety",
        Terrorism => "terrorism",
        ThreatOrViolence => "threat_or_violence",
        SelfHarm => "self_harm",
        Harassment => "harassment",
        Hate => "hate",
        Spam => "spam",
        Privacy => "privacy",
        Copyright => "copyright",
        Other => "other",
    }
}

trust_enum! {
    /// How soon a report or queue item is due.
    ReportPriority {
        Urgent => "urgent",
        Routine => "routine",
    }
}

trust_enum! {
    /// What kind of thing waits in the review queue.
    ReviewItemKind {
        Decision => "decision",
        Report => "report",
    }
}

trust_enum! {
    /// Which side of the queue to list.
    ReviewQueueStatus {
        Open => "open",
        Resolved => "resolved",
    }
}

trust_enum! {
    /// What the operator does with a queue item.
    ReviewAction {
        /// The content may stay (approve or restore).
        Keep => "keep",
        /// The content comes down (takedown).
        Remove => "remove",
    }
}

trust_enum! {
    /// What an entry of the actions feed asks the app to do.
    TrustActionKind {
        Takedown => "takedown",
        Restore => "restore",
        Restriction => "restriction",
        RestrictionLifted => "restriction_lifted",
        Suspend => "suspend",
        Unsuspend => "unsuspend",
    }
}

trust_enum! {
    /// An abuse signal the app may report. `ModerationBlock` and `ReportUpheld`
    /// are recorded by Copepod itself; `record_abuse_signal` refuses them client-side.
    AbuseSignalKind {
        ModerationBlock => "moderation_block",
        ChatJailbreak => "chat_jailbreak",
        ChatHarm => "chat_harm",
        RateLimitHammering => "rate_limit_hammering",
        InviteCap => "invite_cap",
        ReportUpheld => "report_upheld",
    }
}

/// Body of `POST …/reports` as the app's server (API key, scope `trust:write`).
/// Build it with [`ContentReportRequest::new`] and set optional fields on the result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ContentReportRequest {
    /// `kind:id` of the reported thing.
    pub subject_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_url: Option<String>,
    pub category: ReportCategory,
    /// At most 4,000 characters.
    #[serde(default)]
    pub details: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporter_email: Option<String>,
    /// Required on the API-key route: 64 hex characters, a hash of the reporter's address.
    pub reporter_ip_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_owner_user_id: Option<String>,
    /// Captcha token, for servers that verify one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captcha_token: Option<String>,
}

impl ContentReportRequest {
    /// A report with empty details and no optional fields.
    pub fn new(
        subject_ref: impl Into<String>,
        category: ReportCategory,
        reporter_ip_hash: impl Into<String>,
    ) -> Self {
        Self {
            subject_ref: subject_ref.into(),
            subject_url: None,
            category,
            details: String::new(),
            reporter_email: None,
            reporter_ip_hash: reporter_ip_hash.into(),
            subject_owner_user_id: None,
            captcha_token: None,
        }
    }
}

/// Answer of a filed report (`202`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentReportReceipt {
    pub report_id: String,
    /// Always `received`.
    pub status: String,
    pub priority: ReportPriority,
    /// The same reporter reported the same subject within a day.
    pub duplicate: bool,
}

/// Query of the review queue.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewQueueQuery {
    /// Server default is `open`.
    pub status: Option<ReviewQueueStatus>,
    pub kind: Option<ReviewItemKind>,
    /// 1..=200; the server default is 50.
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

/// One row of the queue. It never carries the submitted text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewQueueItem {
    /// `td_…` for a decision, `cr_…` for a report.
    pub id: String,
    pub kind: ReviewItemKind,
    pub purpose: String,
    pub subject_ref: String,
    #[serde(default)]
    pub subject_url: Option<String>,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub engine: Option<String>,
    /// Reports only.
    #[serde(default)]
    pub priority: Option<ReportPriority>,
    pub created: String,
    #[serde(default)]
    pub due_at: Option<String>,
    pub status: ReviewQueueStatus,
}

/// Page of the queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewQueuePage {
    pub items: Vec<ReviewQueueItem>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Body of `POST …/trust/queue/{item_id}/resolve`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveReviewRequest {
    pub action: ReviewAction,
    /// The decision was wrong (decisions only).
    #[serde(default)]
    pub wrong_decision: bool,
    /// At most 500 characters; a report keeps it as its outcome note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Answer of a resolve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveReviewResponse {
    pub id: String,
    pub status: ReviewQueueStatus,
    pub outcome: String,
    pub wrong_decision: bool,
}

/// Query of the admin report list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentReportQuery {
    /// `true` lists resolved reports; the default is open ones.
    pub resolved: bool,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

/// One report in the admin list. `details` is what the reporter wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentReportItem {
    pub id: String,
    pub subject_ref: String,
    #[serde(default)]
    pub subject_url: Option<String>,
    pub category: ReportCategory,
    pub details: String,
    pub priority: ReportPriority,
    /// `open`, `actioned` or `dismissed`.
    pub status: String,
    pub created: String,
    pub due_at: String,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub outcome_note: Option<String>,
}

/// Page of reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentReportPage {
    pub items: Vec<ContentReportItem>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// One entry of the actions feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustAction {
    pub seq: i64,
    pub kind: TrustActionKind,
    #[serde(default)]
    pub subject_ref: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    pub created: String,
}

/// Answer of `GET …/trust/actions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustActionPage {
    pub actions: Vec<TrustAction>,
    /// Pass as `after` next time: the highest `seq` returned, or the `after` sent.
    pub next_after: i64,
}

/// Body of `POST …/trust/signals` (API key, scope `trust:write`). The server sets the weight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbuseSignalRequest {
    pub user_id: String,
    pub kind: AbuseSignalKind,
    /// At most 200 characters of letters, digits and `_ . : / -` (ids, never content).
    #[serde(default)]
    pub evidence_ref: String,
}

/// An active restriction as an app user or an app sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictionView {
    pub scope: RestrictionScope,
    /// RFC 3339 end time; `None` only for `account` (a suspension).
    #[serde(default)]
    pub until: Option<String>,
    pub reason_code: String,
}

/// Answer of a signal (`202`). `restriction` is set when this signal tipped the
/// score over the threshold. `signal_id` is empty when a repeated
/// `rate_limit_hammering` within the hour was not recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbuseSignalReceipt {
    pub signal_id: String,
    #[serde(default)]
    pub restriction: Option<RestrictionView>,
}

/// Answer of `GET …/auth/{collection}/me/restrictions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyRestrictions {
    pub restrictions: Vec<RestrictionView>,
    pub suspended: bool,
}

/// One ledger row in the owner's view. Never carries content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbuseSignalView {
    pub kind: AbuseSignalKind,
    pub weight: f64,
    pub evidence_ref: String,
    pub created: String,
}

/// One restriction row in the owner's view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictionDetail {
    pub scope: RestrictionScope,
    /// `auto` or `owner`.
    pub source: String,
    pub reason: String,
    pub created: String,
    #[serde(default)]
    pub until: Option<String>,
}

/// Decayed abuse score per scope (restriction threshold 2.5).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct AbuseScopeScores {
    pub hosted_ai: f64,
    pub public_posting: f64,
    pub invites: f64,
}

/// A suspension, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspensionInfo {
    pub since: String,
    pub reason: String,
}

/// Answer of `GET …/app-users/{user_id}/trust` (org admin).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppUserTrust {
    pub scores: AbuseScopeScores,
    pub signals: Vec<AbuseSignalView>,
    pub restrictions: Vec<RestrictionDetail>,
    #[serde(default)]
    pub suspended: Option<SuspensionInfo>,
}

/// One row of `GET …/trust/users`: a user with a recent signal or an active restriction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlaggedAppUser {
    pub user_id: String,
    pub scores: AbuseScopeScores,
    pub signals: Vec<AbuseSignalView>,
    pub restrictions: Vec<RestrictionDetail>,
    #[serde(default)]
    pub suspended: Option<SuspensionInfo>,
}

/// Answer of `GET …/trust/users` (at most 100 users).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlaggedAppUsers {
    pub users: Vec<FlaggedAppUser>,
}

/// Answer of suspend (`suspended: true`, `since`) and unsuspend (`suspended: false`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspensionState {
    pub suspended: bool,
    #[serde(default)]
    pub since: Option<String>,
}

/// Details of a 403 `ai_paused_for_review`, read with
/// `CopepodError::ai_paused_for_review_details`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPausedForReview {
    /// Always `hosted_ai` today.
    pub scope: RestrictionScope,
    /// RFC 3339 end of the pause; `None` if the server sent none.
    #[serde(default)]
    pub until: Option<String>,
}

/// Details of a 403 `account_suspended`, read with `CopepodError::account_suspended_details`.
/// Both fields are `None` unless the credentials were proven (password, magic link, MFA).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSuspended {
    /// Single-use token for `app_erase_account`.
    #[serde(default)]
    pub erasure_token: Option<String>,
    /// Seconds the token stays valid (600).
    #[serde(default, rename = "erasure_token_expires_in")]
    pub expires_in: Option<u64>,
}

/// Answer of the erase call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountErased {
    pub deleted: bool,
    #[serde(default)]
    pub erased_rows: u64,
}
