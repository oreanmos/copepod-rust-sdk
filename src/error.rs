use thiserror::Error;

/// Stable Copepod error code returned while no Raft leader can serve a request.
pub const RAFT_LEADER_UNAVAILABLE_CODE: &str = "raft_leader_unavailable";

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
            } if code == RAFT_LEADER_UNAVAILABLE_CODE
        )
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
