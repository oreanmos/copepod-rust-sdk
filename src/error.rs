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

/// 503: a rolling upgrade holds Space writes.
pub const SPACES_UPGRADING_CODE: &str = "spaces_upgrading";

/// 503: a rolling upgrade holds credit reports.
pub const CREDITS_UPGRADING_CODE: &str = "credits_upgrading";

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
            Self::Api { status, .. } | Self::ApiWithDetails { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// The server's stable machine-readable error code, when it sent one.
    pub fn api_code(&self) -> Option<&str> {
        match self {
            Self::Api { code, .. } | Self::ApiWithDetails { code, .. } => code.as_deref(),
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
