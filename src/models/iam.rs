use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRole {
    pub id: String,
    pub org_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRoleCreate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamRoleUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBinding {
    pub id: String,
    pub org_id: String,
    pub role_id: String,
    pub subject_type: String,
    pub subject_id: String,
    #[serde(default)]
    pub app_id: Option<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBindingCreate {
    pub role_id: String,
    pub subject_type: String,
    pub subject_id: String,
    #[serde(default)]
    pub app_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    pub id: String,
    pub org_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub active: bool,
    #[serde(default)]
    pub last_used: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountCreate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}
