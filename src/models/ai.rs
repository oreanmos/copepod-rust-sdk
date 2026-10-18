/// A hosted AI gateway response: Mistral's body plus the `X-Copepod-*` metering headers.
#[derive(Debug, Clone, PartialEq)]
pub struct AiResponse<T> {
    /// The provider body, passed through unchanged.
    pub body: T,
    /// `X-Copepod-Request-Id`.
    pub request_id: Option<String>,
    /// `X-Copepod-Credits-Charged`.
    pub charged_credits: Option<i64>,
    /// `X-Copepod-Credits-Remaining`.
    pub remaining_credits: Option<i64>,
    /// `X-Copepod-Usage-Estimated`: true when the charge was estimated rather than measured.
    pub estimated: bool,
}

/// A multipart audio transcription request.
#[derive(Debug, Clone, PartialEq)]
pub struct AiTranscriptionRequest {
    /// Provider model id (the `model` form field).
    pub model: String,
    /// Raw audio bytes (the `file` part).
    pub bytes: Vec<u8>,
    /// File name sent with the `file` part.
    pub filename: String,
    /// Extra form fields passed through to the provider (never `model` or `file`).
    pub fields: Vec<(String, String)>,
}
