use serde::{Deserialize, Serialize};

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

/// What the provider says a model can do. Every flag defaults to `false`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AiModelCapabilities {
    #[serde(default)]
    pub completion_chat: bool,
    #[serde(default)]
    pub completion_fim: bool,
    #[serde(default)]
    pub function_calling: bool,
    #[serde(default)]
    pub fine_tuning: bool,
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub classification: bool,
}

/// One hosted model the caller's plan offers. Never carries a price.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AiModelInfo {
    /// Provider model id, as sent in a chat request.
    #[serde(default)]
    pub id: String,
    /// Human-readable name.
    #[serde(default)]
    pub display_name: String,
    /// Who made the model, e.g. `Z.ai`.
    #[serde(default)]
    pub vendor: String,
    /// How it is served, e.g. `Z.ai via Mistral`.
    #[serde(default)]
    pub via: String,
    /// Open strings: today `chat`, `analysis`, `vision`, `embedding`, `audio`,
    /// `realtime_audio`, `ocr`, `moderation`. Match what you know, ignore the rest.
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub capabilities: AiModelCapabilities,
    #[serde(default)]
    pub max_context_length: Option<i64>,
    /// Open string: today `low`, `medium` or `high`.
    #[serde(default)]
    pub cost_tier: String,
    /// The provider's deprecation notice, when one exists.
    #[serde(default)]
    pub deprecation: Option<String>,
}

/// A model retired in the last 90 days, so a client can explain a stale saved choice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AiRetiredModel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    /// The announced replacement when the plan offers it, else the plan default.
    /// Calls do not run on it: a retired model falls back to `default_model`.
    #[serde(default)]
    pub replaced_by: Option<String>,
}

/// The hosted models the caller's plan allows right now.
///
/// `hosted_ai_enabled: true` can come with empty `models` and a `null`
/// `default_model` (nothing priced yet); `hosted_ai_enabled: false` always
/// has empty lists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AiModelList {
    #[serde(default)]
    pub hosted_ai_enabled: bool,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub models: Vec<AiModelInfo>,
    #[serde(default)]
    pub retired: Vec<AiRetiredModel>,
}

/// Result of asking for the model list: the list, or a node that predates the route.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AiModelsResponse {
    /// The server answered 200.
    Available(AiModelList),
    /// The server answered 404: a node running an older image has no
    /// `me/ai/models` route. Keep the last list or free-text entry.
    Unavailable,
}
