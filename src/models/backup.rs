use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub app_id: String,
    pub snapshots: Vec<BackupInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupRequest {
    #[serde(default)]
    pub destination_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub snapshot_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDestination {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDestinationsResponse {
    pub destinations: Vec<BackupDestination>,
    #[serde(default)]
    pub default_destination_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDestinationsUpdateRequest {
    pub destinations: Vec<BackupDestination>,
    #[serde(default)]
    pub default_destination_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDiscoverRequest {
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExecuteRequest {
    pub source_path: String,
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlRequest {
    pub key: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlResponse {
    pub url: String,
    #[serde(default)]
    pub expires_at: Option<String>,
}
