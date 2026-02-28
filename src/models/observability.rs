use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorGroup {
    pub id: String,
    pub app_id: String,
    pub fingerprint: String,
    pub message: String,
    pub first_seen: String,
    pub last_seen: String,
    pub count: i64,
    pub status: String,
    #[serde(default)]
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub app_id: String,
    pub name: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub channels: Option<String>,
    pub enabled: bool,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCreate {
    pub name: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub channels: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub channels: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}
