use serde::{Deserialize, Serialize};

/// A registered app migration in the control plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMigration {
    pub id: String,
    pub app_id: String,
    pub version: u32,
    pub name: String,
    pub checksum: String,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

/// A single migration in a sync request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationInput {
    pub version: u32,
    pub name: String,
    pub sql: String,
}

/// Request body for POST .../migrations/sync
#[derive(Debug, Clone, Serialize)]
pub struct MigrationSyncRequest {
    pub migrations: Vec<MigrationInput>,
}

/// Response from the migration sync endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct MigrationSyncResponse {
    pub applied: Vec<u32>,
    pub already_applied: Vec<u32>,
    pub current_version: u32,
}
