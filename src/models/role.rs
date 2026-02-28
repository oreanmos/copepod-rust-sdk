use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An application role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRole {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub app_id: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// A user-role assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: String,
    pub role_id: String,
    #[serde(default)]
    pub role_name: Option<String>,
    pub created: DateTime<Utc>,
}

/// An action entry (permission).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub key: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
}

/// Mapping of an action to a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRoleMapping {
    pub action_key: String,
    pub role_id: String,
    pub allowed: bool,
}
