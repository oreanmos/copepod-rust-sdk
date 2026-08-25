use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AppBackupHealth, AppBackupPolicy, BackupDestinationsResponse, BackupStatus,
    CreateBackupRequest, CreateBackupResponse,
};

impl CopepodClient {
    /// Discover tables from an import source.
    pub async fn import_discover(
        &self,
        org_id: &str,
        app_id: &str,
        body: &CreateBackupRequest,
    ) -> Result<CreateBackupResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{}/apps/{}/import/discover",
                org_id, app_id
            ),
            body,
        )
        .await
    }

    /// Execute an import.
    pub async fn import_execute(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!(
                "api/platform/orgs/{}/apps/{}/import/execute",
                org_id, app_id
            ),
            body,
        )
        .await
    }

    /// Create a backup of an app.
    pub async fn create_backup(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!("api/platform/orgs/{}/apps/{}/backup", org_id, app_id),
            body,
        )
        .await
    }

    /// List available backups for an app.
    pub async fn list_backups(&self, org_id: &str, app_id: &str) -> Result<BackupStatus> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/backups",
            org_id, app_id
        ))
        .await
    }

    /// Read the recurring backup protection policy for an app.
    pub async fn get_backup_policy(&self, org_id: &str, app_id: &str) -> Result<AppBackupPolicy> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/backup-policy",
            org_id, app_id
        ))
        .await
    }

    /// Replace the recurring backup protection policy for an app.
    pub async fn update_backup_policy(
        &self,
        org_id: &str,
        app_id: &str,
        body: &AppBackupPolicy,
    ) -> Result<AppBackupPolicy> {
        self.patch(
            &format!("api/platform/orgs/{}/apps/{}/backup-policy", org_id, app_id),
            body,
        )
        .await
    }

    /// Read exact-destination recovery protection health for an app.
    pub async fn get_backup_health(&self, org_id: &str, app_id: &str) -> Result<AppBackupHealth> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/backup-health",
            org_id, app_id
        ))
        .await
    }

    /// Restore an app from a backup.
    pub async fn restore_backup(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!("api/platform/orgs/{}/apps/{}/restore", org_id, app_id),
            body,
        )
        .await
    }

    /// Get backup destinations configuration.
    pub async fn get_backup_destinations(&self) -> Result<BackupDestinationsResponse> {
        self.get("api/platform/backups/destinations").await
    }

    /// Update backup destinations configuration.
    pub async fn set_backup_destinations(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<BackupDestinationsResponse> {
        self.put("api/platform/backups/destinations", body).await
    }
}
