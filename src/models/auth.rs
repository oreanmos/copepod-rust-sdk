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

/// Response from MFA enrollment endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollResponse {
    pub qr_svg: String,
    pub totp_uri: String,
    pub recovery_codes: Vec<String>,
}

/// Result of an app login attempt — either success or MFA challenge.
#[derive(Debug, Clone)]
pub enum AppLoginResult {
    /// Login succeeded, tokens issued.
    Success(AuthResponse),
    /// MFA is required, caller must verify with the mfa_token.
    MfaRequired(MfaChallenge),
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
