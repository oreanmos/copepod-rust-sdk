use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEnv {
    pub id: String,
    pub app_id: String,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub config: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEnvCreate {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub id: String,
    pub app_id: String,
    pub env: String,
    pub key: String,
    pub value: String,
    pub secret: bool,
    pub version: i32,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntryUpsert {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub env: Option<String>,
}
