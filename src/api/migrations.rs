use std::path::Path;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{AppMigration, MigrationInput, MigrationSyncRequest, MigrationSyncResponse};

impl CopepodClient {
    /// List all registered migrations for an app.
    pub async fn list_migrations(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Vec<AppMigration>> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/migrations",
            org_id, app_id
        ))
        .await
    }

    /// Sync migrations from a list of inputs. Registers new migrations and
    /// applies any that haven't been applied yet.
    pub async fn sync_migrations(
        &self,
        org_id: &str,
        app_id: &str,
        migrations: &[MigrationInput],
    ) -> Result<MigrationSyncResponse> {
        let req = MigrationSyncRequest {
            migrations: migrations.to_vec(),
        };
        self.post(
            &format!(
                "api/platform/orgs/{}/apps/{}/migrations/sync",
                org_id, app_id
            ),
            &req,
        )
        .await
    }

    /// Read all `.sql` files from a directory, assign sequential version numbers
    /// based on filename sort order, and sync them to copepod.
    ///
    /// Files are sorted lexicographically by name, so both `001_foo.sql` and
    /// `20260215000001_create_notes.sql` patterns work correctly.
    pub async fn sync_migrations_dir(
        &self,
        org_id: &str,
        app_id: &str,
        dir: &Path,
    ) -> Result<MigrationSyncResponse> {
        let migrations = read_migrations_dir(dir)?;
        self.sync_migrations(org_id, app_id, &migrations).await
    }
}

/// Read `.sql` files from a directory and convert them to sequential `MigrationInput`s.
fn read_migrations_dir(dir: &Path) -> Result<Vec<MigrationInput>> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| CopepodError::Io(format!("failed to read migrations dir: {e}")))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sql") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    entries.sort();

    let mut migrations = Vec::with_capacity(entries.len());
    for (i, path) in entries.iter().enumerate() {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let sql = std::fs::read_to_string(path).map_err(|e| {
            CopepodError::Io(format!(
                "failed to read migration file {}: {e}",
                path.display()
            ))
        })?;
        migrations.push(MigrationInput {
            version: (i as u32) + 1,
            name,
            sql,
        });
    }

    Ok(migrations)
}
