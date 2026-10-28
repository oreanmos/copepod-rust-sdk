use thiserror::Error;

/// Stable Copepod error code returned while no Raft leader can serve a request.
pub const RAFT_LEADER_UNAVAILABLE_CODE: &str = "raft_leader_unavailable";

/// Deploy refused with 409: a single replica has no room; retry with `allow_outage`.
pub const ROLLOUT_NEEDS_OUTAGE_CODE: &str = "rollout_needs_outage";

/// Deploy refused with 409: a blue-green rollout has no room on the cluster.
pub const ROLLOUT_NEEDS_CAPACITY_CODE: &str = "rollout_needs_capacity";

/// Checkout refused with 409: an earlier checkout is still open. The body
/// carries `checkout_url` and `expires_at` at its top level.
pub const CHECKOUT_PENDING_CODE: &str = "checkout_pending";

/// 409 on an `If-Match` update: the record changed. The body carries
/// `current_rev` at its top level.
pub const REVISION_CONFLICT_CODE: &str = "revision_conflict";

/// 402 on credits check, 409 on credits report: nothing left to spend. The
/// body carries `remaining_total` and `resets_at` at its top level.
pub const CREDITS_EXHAUSTED_CODE: &str = "credits_exhausted";

/// 402 on a hosted AI call: no credits left. The body carries `remaining_total`
/// and `resets_at` at its top level.
pub const AI_CREDITS_EXHAUSTED_CODE: &str = "ai_credits_exhausted";

/// 409 on a hosted AI call: this `Idempotency-Key` was already charged (or is still
/// running). The body carries `charged_credits` at its top level.
pub const AI_REQUEST_REPLAYED_CODE: &str = "ai_request_replayed";

/// 400 on Laya `decide`: the body is neither a task nor an ad-hoc request, or breaks a limit.
pub const LAYA_INVALID_REQUEST_CODE: &str = "laya_invalid_request";

/// 403 on Laya `decide`: an ad-hoc question set on the app-user route. Ad-hoc is API-key only.
pub const LAYA_ADHOC_FORBIDDEN_CODE: &str = "laya_adhoc_forbidden";

/// 404 on Laya `decide`: no such task. (A node without the route answers 404 with no code.)
pub const LAYA_TASK_UNKNOWN_CODE: &str = "laya_task_unknown";

/// 404 on a decide outcome: the decision belongs to another app, is missing, or is older than 180 days.
pub const LAYA_DECISION_UNKNOWN_CODE: &str = "laya_decision_unknown";

/// 400 on a decide outcome or a donation: a label names an unknown question or option, or there are none.
pub const LAYA_INVALID_LABEL_CODE: &str = "laya_invalid_label";

/// 503 on Laya `decide`: timeout, busy, not ready or circuit open. Treat it as "no answer".
pub const LAYA_UNAVAILABLE_CODE: &str = "laya_unavailable";

/// 503 on a decide outcome or a donation write: held while a rolling upgrade reaches every peer.
pub const LAYA_UPGRADING_CODE: &str = "laya_upgrading";

/// 403 on a hosted AI call (Laya `decide`, the gateway): the user has an active `hosted_ai`
/// restriction. The body carries `scope` and `until`.
pub const AI_PAUSED_FOR_REVIEW_CODE: &str = "ai_paused_for_review";

/// 403 on any app-user call: the account is suspended by the app's owner.
pub const ACCOUNT_SUSPENDED_CODE: &str = "account_suspended";

/// 429 on a content report: the reporter or the subject hit a report limit.
pub const REPORT_RATE_LIMITED_CODE: &str = "report_rate_limited";

/// 429 on `moderate`: the app or the user hit the moderation rate limit.
pub const MODERATION_RATE_LIMITED_CODE: &str = "moderation_rate_limited";

/// 422 on a record edit: moderation blocked the edited text; nothing was stored.
pub const CONTENT_REJECTED_CODE: &str = "content_rejected";

/// 409 on suspend: the user is already suspended.
pub const ALREADY_SUSPENDED_CODE: &str = "already_suspended";

/// 409 on unsuspend: the user is not suspended.
pub const NOT_SUSPENDED_CODE: &str = "not_suspended";

/// 404 on lifting a restriction: no active restriction in that scope.
pub const RESTRICTION_NOT_FOUND_CODE: &str = "restriction_not_found";

/// 400 on a consent grant: the policy version is not one the server knows.
pub const LAYA_POLICY_VERSION_UNKNOWN_CODE: &str = "laya_policy_version_unknown";

/// 403 on a donation: the user has not turned donations on.
pub const LAYA_DONATION_CONSENT_REQUIRED_CODE: &str = "laya_donation_consent_required";

/// 400 on a donation: the task does not take donations.
pub const LAYA_TASK_NOT_DONATABLE_CODE: &str = "laya_task_not_donatable";

/// 429 on a donation: 2,000 donations in the last 30 days.
pub const LAYA_DONATION_QUOTA_CODE: &str = "laya_donation_quota";

/// 404: the Space does not exist or the caller is not a member.
pub const SPACE_NOT_FOUND_CODE: &str = "space_not_found";

/// 403: the caller's role in the Space cannot do the operation.
pub const INSUFFICIENT_ROLE_CODE: &str = "insufficient_role";

/// 400: a space-scoped collection was used without `X-Space-Id`.
pub const SPACE_REQUIRED_CODE: &str = "space_required";

/// 409: removing or demoting the last owner of a Space.
pub const LAST_OWNER_CODE: &str = "last_owner";

/// 409: accepting an invite for a Space the caller already belongs to.
pub const ALREADY_MEMBER_CODE: &str = "already_member";

/// 403: sending or accepting an invite needs a verified email.
pub const EMAIL_NOT_VERIFIED_CODE: &str = "email_not_verified";

/// 403: the invite was sent to another email address.
pub const INVITE_EMAIL_MISMATCH_CODE: &str = "invite_email_mismatch";

/// 410: the invite is older than 14 days.
pub const INVITE_EXPIRED_CODE: &str = "invite_expired";

/// 404: unknown, revoked or already accepted invite, or its Space is deleted.
pub const INVITE_NOT_FOUND_CODE: &str = "invite_not_found";

/// 429: daily invite limits reached.
pub const INVITE_RATE_LIMITED_CODE: &str = "invite_rate_limited";

/// 429: too many Spaces created today.
pub const SPACE_RATE_LIMITED_CODE: &str = "space_rate_limited";

/// 429: too many suggestions to a commons, or too many waiting.
pub const SUGGESTION_RATE_LIMITED_CODE: &str = "suggestion_rate_limited";

/// 404: unknown suggestion.
pub const SUGGESTION_NOT_FOUND_CODE: &str = "suggestion_not_found";

/// 409: the suggestion was already accepted, rejected or withdrawn.
pub const SUGGESTION_DECIDED_CODE: &str = "suggestion_decided";

/// 400: the Space is not a commons (suggestions, takedown).
pub const NOT_COMMONS_CODE: &str = "not_commons";

/// 409: the app's master switch for anonymous commons reads is off, so an
/// owner cannot switch `anonymous_read` on.
pub const ANONYMOUS_COMMONS_DISABLED_CODE: &str = "anonymous_commons_disabled";

/// 503: a rolling upgrade holds back switching anonymous commons reads on.
pub const ANONYMOUS_COMMONS_UPGRADING_CODE: &str = "anonymous_commons_upgrading";

/// 503: a rolling upgrade holds commons writes.
pub const COMMONS_UPGRADING_CODE: &str = "commons_upgrading";

/// 503: account deletion could not finish re-attributing commons records;
/// repeating the deletion finishes them.
pub const COMMONS_INCOMPLETE_CODE: &str = "commons_incomplete";

/// 503: a rolling upgrade holds share link writes.
pub const SHARE_LINKS_UPGRADING_CODE: &str = "share_links_upgrading";

/// 404: unknown share link or token.
pub const LINK_NOT_FOUND_CODE: &str = "link_not_found";

/// 410: the share link was revoked or rotated away.
pub const LINK_REVOKED_CODE: &str = "link_revoked";

/// 410: the share link is past its expiry.
pub const LINK_EXPIRED_CODE: &str = "link_expired";

/// 429: too many failed link lookups from one address.
pub const LINK_RATE_LIMITED_CODE: &str = "link_rate_limited";

/// 429: too many share links created today.
pub const SHARE_LINK_RATE_LIMITED_CODE: &str = "share_link_rate_limited";

/// 409: the Space holds the most share links; revoke one first.
pub const SHARE_LINK_LIMIT_CODE: &str = "share_link_limit";

/// 503: a rolling upgrade holds Space writes.
pub const SPACES_UPGRADING_CODE: &str = "spaces_upgrading";

/// 503: a rolling upgrade holds credit reports.
pub const CREDITS_UPGRADING_CODE: &str = "credits_upgrading";

/// 402 on a storage pack subscribe: the user has no active, non-trial paid plan.
pub const REQUIRES_PAID_PLAN_CODE: &str = "requires_paid_plan";

/// 409 on a storage pack subscribe: the pack's currency is not the plan's.
pub const CURRENCY_MISMATCH_CODE: &str = "currency_mismatch";

/// 409 on a storage pack subscribe: the plan already has unlimited storage.
pub const STORAGE_UNLIMITED_CODE: &str = "storage_unlimited";

/// 409 on a storage pack subscribe: the app has no spare storage capacity.
pub const STORAGE_CAPACITY_UNAVAILABLE_CODE: &str = "capacity_unavailable";

/// 409 on a storage pack subscribe: the payment provider has no usable mandate.
/// The same idempotency key can be retried later.
pub const MANDATE_UNAVAILABLE_CODE: &str = "mandate_unavailable";

/// 502 on a storage pack subscribe: the payment provider refused the subscription.
pub const PAYMENT_PROVIDER_REJECTED_CODE: &str = "payment_provider_rejected";

/// 503: a rolling upgrade holds storage pack subscribe, cancel and settlement.
pub const PACKS_UPGRADING_CODE: &str = "packs_upgrading";

/// 409 at signup or billing intent: the app is at capacity. The body carries `limit`
/// (the full capacity setting, e.g. `max_active_users`). Do not retry.
pub const CAPACITY_REACHED_CODE: &str = "capacity_reached";

/// 422 on an upload: the user's storage quota would be exceeded. The body carries
/// `used_bytes`, `limit_bytes` and `file_bytes`. Do not retry.
pub const STORAGE_QUOTA_EXCEEDED_CODE: &str = "storage_quota_exceeded";

/// Errors returned by the Copepod SDK.
#[derive(Debug, Error)]
pub enum CopepodError {
    /// HTTP transport error from reqwest.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API error returned by the Copepod server.
    #[error("API error {status}: {message}")]
    Api {
        status: u16,
        code: Option<String>,
        message: String,
    },

    /// The deploy endpoint's `rollout_needs_outage` / `rollout_needs_capacity`
    /// conflict with its structured `details`, and the billing
    /// `checkout_pending` conflict, whose `details` is the whole response body
    /// (its fields sit at the top level). Every other error, with or
    /// without details, stays [`CopepodError::Api`]; use [`CopepodError::api_status`],
    /// [`CopepodError::api_code`] and [`CopepodError::api_details`] to read
    /// both shapes uniformly.
    #[error("API error {status}: {message}")]
    ApiWithDetails {
        status: u16,
        code: Option<String>,
        message: String,
        details: serde_json::Value,
    },

    /// 409 `capacity_reached`: the app is at capacity. Not retryable; offer a
    /// waitlist. Falls back to [`CopepodError::Api`] when `limit` is missing.
    #[error("API error {status}: {message}")]
    CapacityReached {
        status: u16,
        message: String,
        /// The capacity setting that is full, e.g. `max_active_users`.
        limit: String,
    },

    /// 422 `storage_quota_exceeded`: the upload would pass the user's quota. Not
    /// retryable. Falls back to [`CopepodError::Api`] when any number is missing.
    #[error("API error {status}: {message}")]
    StorageQuotaExceeded {
        status: u16,
        message: String,
        used_bytes: u64,
        limit_bytes: u64,
        file_bytes: u64,
    },

    /// Authentication error (missing token, expired, etc.).
    #[error("Auth error: {0}")]
    Auth(String),

    /// A client-side request argument failed validation.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// JSON deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// URL parse error.
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    /// Server-sent events error.
    #[error("SSE error: {0}")]
    Sse(String),

    /// Filesystem I/O error (e.g. reading migration files).
    #[error("IO error: {0}")]
    Io(String),
}

impl CopepodError {
    /// Whether the server has no such route: a 404 with an empty body (axum's answer on a node
    /// older than the route), arriving as `Api { status: 404, code: None, message: "Unknown error" }`.
    /// A Copepod JSON 404 (`laya_task_unknown`, unknown app, ...) is not a missing route.
    pub fn is_route_missing(&self) -> bool {
        matches!(
            self,
            Self::Api { status: 404, code: None, message } if message == "Unknown error"
        )
    }

    /// Whether the server rejected this request at the Raft leader interlock.
    pub fn is_raft_leader_unavailable(&self) -> bool {
        matches!(
            self,
            Self::Api {
                status: 503,
                code: Some(code),
                ..
            } | Self::ApiWithDetails {
                status: 503,
                code: Some(code),
                ..
            } if code == RAFT_LEADER_UNAVAILABLE_CODE
        )
    }

    /// HTTP status of an API error.
    pub fn api_status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. }
            | Self::ApiWithDetails { status, .. }
            | Self::CapacityReached { status, .. }
            | Self::StorageQuotaExceeded { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// The server's stable machine-readable error code, when it sent one.
    pub fn api_code(&self) -> Option<&str> {
        match self {
            Self::Api { code, .. } | Self::ApiWithDetails { code, .. } => code.as_deref(),
            Self::CapacityReached { .. } => Some(CAPACITY_REACHED_CODE),
            Self::StorageQuotaExceeded { .. } => Some(STORAGE_QUOTA_EXCEEDED_CODE),
            _ => None,
        }
    }

    /// The server's structured `details` object, when the error carried one.
    pub fn api_details(&self) -> Option<&serde_json::Value> {
        match self {
            Self::ApiWithDetails { details, .. } => Some(details),
            _ => None,
        }
    }

    /// Typed capacity details of a deploy refused with `rollout_needs_outage`
    /// or `rollout_needs_capacity`.
    pub fn rollout_capacity_details(&self) -> Option<crate::models::RolloutCapacityDetails> {
        match self.api_code() {
            Some(ROLLOUT_NEEDS_OUTAGE_CODE | ROLLOUT_NEEDS_CAPACITY_CODE) => {
                serde_json::from_value(self.api_details()?.clone()).ok()
            }
            _ => None,
        }
    }

    /// Typed details of a checkout refused with 409 `checkout_pending`.
    pub fn checkout_pending_details(&self) -> Option<crate::models::CheckoutPending> {
        match self.api_code() {
            Some(CHECKOUT_PENDING_CODE) => serde_json::from_value(self.api_details()?.clone()).ok(),
            _ => None,
        }
    }

    /// Typed details of a 409 `revision_conflict`: the stored revision.
    pub fn revision_conflict_details(&self) -> Option<crate::models::RevisionConflict> {
        match self.api_code() {
            Some(REVISION_CONFLICT_CODE) => {
                serde_json::from_value(self.api_details()?.clone()).ok()
            }
            _ => None,
        }
    }

    /// Whether this is a 409 `revision_conflict`.
    pub fn is_revision_conflict(&self) -> bool {
        self.api_code() == Some(REVISION_CONFLICT_CODE)
    }

    /// Typed details of `credits_exhausted` (402 on check, 409 on report).
    pub fn credits_exhausted_details(&self) -> Option<crate::models::CreditsExhausted> {
        match self.api_code() {
            Some(CREDITS_EXHAUSTED_CODE) => {
                serde_json::from_value(self.api_details()?.clone()).ok()
            }
            _ => None,
        }
    }

    /// Whether the user is out of credits (`credits_exhausted`).
    pub fn is_credits_exhausted(&self) -> bool {
        self.api_code() == Some(CREDITS_EXHAUSTED_CODE)
    }

    /// Whether the Space is missing or the caller is not a member.
    pub fn is_space_not_found(&self) -> bool {
        self.api_code() == Some(SPACE_NOT_FOUND_CODE)
    }

    /// Whether the caller's Space role cannot do the operation.
    pub fn is_insufficient_role(&self) -> bool {
        self.api_code() == Some(INSUFFICIENT_ROLE_CODE)
    }

    /// The full capacity setting of a 409 `capacity_reached`.
    pub fn capacity_reached_limit(&self) -> Option<&str> {
        match self {
            Self::CapacityReached { limit, .. } => Some(limit),
            _ => None,
        }
    }

    /// Whether the app is at capacity (`capacity_reached`).
    pub fn is_capacity_reached(&self) -> bool {
        matches!(self, Self::CapacityReached { .. })
    }

    /// `(used_bytes, limit_bytes, file_bytes)` of a 422 `storage_quota_exceeded`.
    pub fn storage_quota_exceeded_bytes(&self) -> Option<(u64, u64, u64)> {
        match self {
            Self::StorageQuotaExceeded {
                used_bytes,
                limit_bytes,
                file_bytes,
                ..
            } => Some((*used_bytes, *limit_bytes, *file_bytes)),
            _ => None,
        }
    }

    /// Whether an upload was refused for storage quota (`storage_quota_exceeded`).
    pub fn is_storage_quota_exceeded(&self) -> bool {
        matches!(self, Self::StorageQuotaExceeded { .. })
    }

    /// Whether the app-user's account is suspended (403 `account_suspended`).
    pub fn is_account_suspended(&self) -> bool {
        self.api_status() == Some(403) && self.api_code() == Some(ACCOUNT_SUSPENDED_CODE)
    }

    /// Whether hosted AI is paused for this user (403 `ai_paused_for_review`).
    pub fn is_ai_paused_for_review(&self) -> bool {
        self.api_code() == Some(AI_PAUSED_FOR_REVIEW_CODE)
    }

    /// The restriction scope and end time of a 403 `ai_paused_for_review`. `until` is the
    /// RFC 3339 time the pause ends by itself.
    pub fn ai_paused_for_review_details(&self) -> Option<crate::models::AiPausedForReview> {
        match self.api_code() {
            Some(AI_PAUSED_FOR_REVIEW_CODE) => {
                serde_json::from_value(self.api_details()?.clone()).ok()
            }
            _ => None,
        }
    }

    /// Whether a content report was rate limited (429 `report_rate_limited`).
    pub fn is_report_rate_limited(&self) -> bool {
        self.api_code() == Some(REPORT_RATE_LIMITED_CODE)
    }

    /// Whether `moderate` was rate limited (429 `moderation_rate_limited`).
    pub fn is_moderation_rate_limited(&self) -> bool {
        self.api_code() == Some(MODERATION_RATE_LIMITED_CODE)
    }

    /// Whether moderation blocked a record edit (422 `content_rejected`); nothing was stored.
    pub fn is_content_rejected(&self) -> bool {
        self.api_code() == Some(CONTENT_REJECTED_CODE)
    }

    /// Whether suspend found the user already suspended (409 `already_suspended`).
    pub fn is_already_suspended(&self) -> bool {
        self.api_code() == Some(ALREADY_SUSPENDED_CODE)
    }

    /// Whether unsuspend found the user not suspended (409 `not_suspended`).
    pub fn is_not_suspended(&self) -> bool {
        self.api_code() == Some(NOT_SUSPENDED_CODE)
    }

    /// Whether a lift found no active restriction (404 `restriction_not_found`).
    pub fn is_restriction_not_found(&self) -> bool {
        self.api_code() == Some(RESTRICTION_NOT_FOUND_CODE)
    }

    /// Server-advertised retry delay for a Raft leader election.
    ///
    /// This is a hint for caller-controlled retries. The SDK deliberately does
    /// not replay requests automatically because a mutation may not be
    /// idempotent even when the transport response is retryable.
    pub fn raft_retry_after(&self) -> Option<std::time::Duration> {
        self.is_raft_leader_unavailable()
            .then(|| std::time::Duration::from_secs(1))
    }
}

pub type Result<T> = std::result::Result<T, CopepodError>;

#[cfg(test)]
mod tests {
    use super::{CopepodError, RAFT_LEADER_UNAVAILABLE_CODE};

    #[test]
    fn identifies_only_the_stable_raft_leader_error() {
        let unavailable = CopepodError::Api {
            status: 503,
            code: Some(RAFT_LEADER_UNAVAILABLE_CODE.into()),
            message: "Raft leader unavailable; retry shortly".into(),
        };
        assert!(unavailable.is_raft_leader_unavailable());
        assert_eq!(
            unavailable.raft_retry_after(),
            Some(std::time::Duration::from_secs(1))
        );

        let unrelated = CopepodError::Api {
            status: 503,
            code: Some("deployment_backend_unavailable".into()),
            message: "deployment backend unavailable".into(),
        };
        assert!(!unrelated.is_raft_leader_unavailable());
        assert_eq!(unrelated.raft_retry_after(), None);
    }
}
