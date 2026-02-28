use thiserror::Error;

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

    /// JSON deserialization error.
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// URL parse error.
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    /// Server-sent events error.
    #[error("SSE error: {0}")]
    Sse(String),
}

pub type Result<T> = std::result::Result<T, CopepodError>;
