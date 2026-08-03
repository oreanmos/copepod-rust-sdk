use serde::{Deserialize, Serialize};

/// Explicit instance-wide administrator assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceAdministrator {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub created: String,
    pub updated: String,
}

/// List response for explicit instance administrators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceAdministratorsResponse {
    pub items: Vec<InstanceAdministrator>,
}

/// Grant or change an explicit instance role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetInstanceRoleRequest {
    pub role: String,
    pub reason: String,
}

/// Revoke an explicit instance role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeInstanceRoleRequest {
    pub reason: String,
}

/// Result of granting or changing an instance role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRoleResponse {
    pub user_id: String,
    pub role: String,
}

/// Result of revoking an instance role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeInstanceRoleResponse {
    pub user_id: String,
    pub revoked: bool,
}

/// Break-glass request to promote an existing active user to instance owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverInstanceOwnerRequest {
    pub user_id: String,
    pub reason: String,
}

/// Result of a break-glass owner recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverInstanceOwnerResponse {
    pub user_id: String,
    pub role: String,
    pub recovered: bool,
}

/// Durable progress for an envelope-encryption root-key rotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKeyRotationState {
    pub from_fingerprint: String,
    pub to_fingerprint: String,
    pub status: String,
    pub rotated_values: u64,
    pub verified_values: u64,
    #[serde(default)]
    pub started_by: Option<String>,
    pub started: String,
    pub updated: String,
    #[serde(default)]
    pub completed: Option<String>,
}

/// Request a staged root-key rotation action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateSecretsRequest {
    pub action: String,
    pub confirm_fingerprint: String,
}

/// Current root-key rotation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRotationStatusResponse {
    pub primary_fingerprint: String,
    pub previous_configured: bool,
    pub safe_to_remove_previous: bool,
    #[serde(default)]
    pub rotation: Option<SecretKeyRotationState>,
}

/// Result of rotating or retiring a root key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRotationResponse {
    pub rotation: SecretKeyRotationState,
    pub primary_fingerprint: String,
    pub safe_to_remove_previous: bool,
}
