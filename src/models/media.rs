use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub storage_driver: String,
    #[serde(default)]
    pub config: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketCreate {
    pub name: String,
    #[serde(default)]
    pub storage_driver: Option<String>,
    #[serde(default)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnRule {
    pub id: String,
    pub app_id: String,
    #[serde(default)]
    pub cache_control: Option<String>,
    #[serde(default)]
    pub allowed_origins: Option<String>,
    #[serde(default)]
    pub custom_headers: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnRuleUpdate {
    #[serde(default)]
    pub cache_control: Option<String>,
    #[serde(default)]
    pub allowed_origins: Option<String>,
    #[serde(default)]
    pub custom_headers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaJob {
    pub id: String,
    pub app_id: String,
    pub source_key: String,
    pub operation: String,
    #[serde(default)]
    pub params: Option<String>,
    #[serde(default)]
    pub output_key: Option<String>,
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    pub created: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaJobCreate {
    pub source_key: String,
    pub operation: String,
    #[serde(default)]
    pub params: Option<String>,
}
