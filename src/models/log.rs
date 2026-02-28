use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An action log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLog {
    pub id: String,
    pub action: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(default)]
    pub record_id: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    pub created: DateTime<Utc>,
}

/// Aggregated log statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total: u64,
    #[serde(default)]
    pub by_action: Option<serde_json::Value>,
    #[serde(default)]
    pub by_user: Option<serde_json::Value>,
}

/// Filter parameters for log queries.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LogFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}
