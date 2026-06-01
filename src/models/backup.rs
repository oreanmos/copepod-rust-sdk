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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageTransformRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
}

impl ImageTransformRequest {
    pub fn to_query_string(&self) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if let Some(width) = self.width {
            serializer.append_pair("w", &width.to_string());
        }
        if let Some(height) = self.height {
            serializer.append_pair("h", &height.to_string());
        }
        if let Some(fit) = self.fit.as_deref() {
            serializer.append_pair("fit", fit);
        }
        if let Some(format) = self.format.as_deref() {
            serializer.append_pair("format", format);
        }
        if let Some(quality) = self.quality {
            serializer.append_pair("q", &quality.to_string());
        }
        serializer.finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlRequest {
    pub key: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<ImageTransformRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlResponse {
    pub url: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub expires: Option<u64>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub transform: Option<String>,
}
