use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInvite {
    pub id: String,
    pub org_id: String,
    pub email: String,
    pub role: String,
    pub invited_by: String,
    pub status: String,
    #[serde(default)]
    pub expires: Option<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInviteCreate {
    pub email: String,
    pub role: String,
}
