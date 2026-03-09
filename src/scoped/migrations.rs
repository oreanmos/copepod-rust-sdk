use std::path::Path;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AppMigration, MigrationInput, MigrationSyncResponse};

/// Migration helpers bound to a specific app.
#[derive(Debug, Clone)]
pub struct ScopedMigrationClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedMigrationClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: &str, app_id: &str) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// List registered migrations for this app.
    pub async fn list(&self) -> Result<Vec<AppMigration>> {
        self.client
            .list_migrations(&self.org_id, &self.app_id)
            .await
    }

    /// Sync migration inputs for this app.
    pub async fn sync(&self, migrations: &[MigrationInput]) -> Result<MigrationSyncResponse> {
        self.client
            .sync_migrations(&self.org_id, &self.app_id, migrations)
            .await
    }

    /// Read and sync all `.sql` files from a directory for this app.
    pub async fn sync_dir(&self, dir: &Path) -> Result<MigrationSyncResponse> {
        self.client
            .sync_migrations_dir(&self.org_id, &self.app_id, dir)
            .await
    }
}
