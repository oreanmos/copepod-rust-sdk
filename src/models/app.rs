use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An application within an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    pub org_id: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Request body to create an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCreate {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
}

/// Request body to update an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
}

/// An API key for an application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub key_prefix: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub created: DateTime<Utc>,
}
