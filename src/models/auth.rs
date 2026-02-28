use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Successful authentication response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: User,
}

/// MFA challenge returned when MFA is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaChallenge {
    pub mfa_required: bool,
    pub mfa_token: String,
}

/// A platform or app user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub avatar: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
