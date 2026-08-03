use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    ManagedDatabaseBackupDeleteResponse, ManagedDatabaseBackupsResponse, ManagedDatabaseRestoreJob,
    RestoreDatabaseBackupRequest, TriggerDatabaseBackupRequest, VolumeBackup,
};

impl CopepodClient {
    /// List backup artifacts and restore jobs for a managed database.
    pub async fn list_managed_database_backups(
        &self,
        org_id: &str,
        deployment_id: &str,
    ) -> Result<ManagedDatabaseBackupsResponse> {
        self.get(&format!(
            "api/platform/orgs/{org_id}/databases/{deployment_id}/backups"
        ))
        .await
    }

    /// Queue a streamed backup to an enabled S3 or SFTP destination.
    pub async fn trigger_managed_database_backup(
        &self,
        org_id: &str,
        deployment_id: &str,
        request: &TriggerDatabaseBackupRequest,
    ) -> Result<VolumeBackup> {
        self.post(
            &format!("api/platform/orgs/{org_id}/databases/{deployment_id}/backups"),
            request,
        )
        .await
    }

    /// Queue a checksum-verified restore with a distinct safety backup.
    pub async fn restore_managed_database_backup(
        &self,
        org_id: &str,
        deployment_id: &str,
        backup_id: &str,
        request: &RestoreDatabaseBackupRequest,
    ) -> Result<ManagedDatabaseRestoreJob> {
        self.post(
            &format!(
                "api/platform/orgs/{org_id}/databases/{deployment_id}/backups/{backup_id}/restore"
            ),
            request,
        )
        .await
    }

    /// Delete an unreferenced terminal backup and its remote artifact.
    pub async fn delete_managed_database_backup(
        &self,
        org_id: &str,
        deployment_id: &str,
        backup_id: &str,
    ) -> Result<ManagedDatabaseBackupDeleteResponse> {
        let response = self
            .auth_request(
                reqwest::Method::DELETE,
                &format!(
                    "api/platform/orgs/{org_id}/databases/{deployment_id}/backups/{backup_id}"
                ),
            )
            .await?
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }
}
