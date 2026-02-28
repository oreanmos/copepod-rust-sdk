use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub org_id: String,
    pub resource_type: String,
    pub retention_days: i32,
    pub action: String,
    pub enabled: bool,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyCreate {
    pub resource_type: String,
    pub retention_days: i32,
    #[serde(default)]
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyUpdate {
    #[serde(default)]
    pub retention_days: Option<i32>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub id: String,
    pub org_id: String,
    #[serde(default)]
    pub app_id: Option<String>,
    pub scope: String,
    pub format: String,
    pub status: String,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub requested_by: String,
    pub created: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJobCreate {
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsarRequest {
    pub id: String,
    pub org_id: String,
    pub request_type: String,
    pub subject_email: String,
    #[serde(default)]
    pub subject_id: Option<String>,
    pub status: String,
    #[serde(default)]
    pub result: Option<String>,
    pub requested_by: String,
    pub created: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsarRequestCreate {
    pub request_type: String,
    pub subject_email: String,
    #[serde(default)]
    pub subject_id: Option<String>,
}
