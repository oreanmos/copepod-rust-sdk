use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    #[serde(default)]
    pub id: String,
    pub filename: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub destination: String,
    #[serde(default)]
    pub destination_meta: serde_json::Value,
    #[serde(default)]
    pub shard_count: i64,
    #[serde(default)]
    pub file_count: i64,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub app_id: String,
    pub snapshots: Vec<BackupInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateBackupRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destination_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_id: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub snapshot_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupResponse {
    pub status: String,
    pub backup_id: String,
    pub filename: String,
    pub destination: serde_json::Value,
    pub shard_count: i64,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBackupPolicy {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub interval_hours: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_minutes: Option<u32>,
    #[serde(default)]
    pub retention_days: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_id: Option<String>,
    #[serde(default)]
    pub destination_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBackupHealth {
    pub status: String,
    pub enabled: bool,
    pub target_rpo_minutes: u32,
    pub latest_recovery_point_at: Option<String>,
    pub age_minutes: Option<i64>,
    pub required_destinations: usize,
    pub successful_destinations: usize,
    #[serde(default)]
    pub required_destination_ids: Vec<String>,
    #[serde(default)]
    pub successful_destination_ids: Vec<String>,
    pub latest_attempt_at: Option<String>,
    pub latest_attempt_status: Option<String>,
    pub latest_attempt_reason: Option<String>,
    pub reason: Option<String>,
}

/// Status of a durable managed-database backup or restore job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VolumeBackupStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    #[serde(other)]
    Unknown,
}

/// A streamed, remote managed-database backup artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeBackup {
    pub id: String,
    pub volume_id: String,
    pub deployed_app_id: String,
    pub filename: String,
    pub size_bytes: i64,
    pub status: VolumeBackupStatus,
    pub destination: String,
    pub destination_meta: serde_json::Value,
    #[serde(default)]
    pub checksum_sha256: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// Durable status of a managed-database restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedDatabaseRestoreJob {
    pub id: String,
    pub backup_id: String,
    pub safety_backup_id: String,
    pub deployed_app_id: String,
    pub status: VolumeBackupStatus,
    #[serde(default)]
    pub error_message: Option<String>,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// Backup artifacts and restore jobs for a managed database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedDatabaseBackupsResponse {
    pub items: Vec<VolumeBackup>,
    pub restore_jobs: Vec<ManagedDatabaseRestoreJob>,
}

/// Request a managed-database backup using an explicit or default destination.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerDatabaseBackupRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_id: Option<String>,
}

/// Request a checksum-verified restore with a separate recovery point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreDatabaseBackupRequest {
    pub confirmation: String,
    pub safety_backup_id: String,
}

impl RestoreDatabaseBackupRequest {
    /// Build the exact confirmation required by the restore endpoint.
    pub fn confirmed(backup_id: impl Into<String>, safety_backup_id: impl Into<String>) -> Self {
        let backup_id = backup_id.into();
        Self {
            confirmation: format!("restore:{backup_id}"),
            safety_backup_id: safety_backup_id.into(),
        }
    }
}

/// Response returned after deleting a managed-database backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedDatabaseBackupDeleteResponse {
    pub deleted: bool,
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

#[cfg(test)]
mod tests {
    use super::{CreateBackupRequest, RestoreDatabaseBackupRequest, VolumeBackupStatus};

    #[test]
    fn managed_database_status_and_confirmation_match_the_wire_contract() {
        assert_eq!(
            serde_json::to_string(&VolumeBackupStatus::InProgress).unwrap(),
            "\"in_progress\""
        );
        let request = RestoreDatabaseBackupRequest::confirmed("backup-1", "safety-2");
        assert_eq!(request.confirmation, "restore:backup-1");
        assert_eq!(request.safety_backup_id, "safety-2");
    }

    #[test]
    fn app_backup_request_serializes_the_exact_destination_set() {
        let request = CreateBackupRequest {
            destination_id: None,
            destination_ids: vec!["primary-s3".to_string(), "secondary-sftp".to_string()],
        };
        let value = serde_json::to_value(request).unwrap();
        assert!(value.get("destination_id").is_none());
        assert_eq!(
            value["destination_ids"],
            serde_json::json!(["primary-s3", "secondary-sftp"])
        );
    }
}
