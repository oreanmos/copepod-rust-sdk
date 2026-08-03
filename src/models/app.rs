use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Read records through an app API key.
pub const API_KEY_SCOPE_RECORDS_READ: &str = "records:read";
/// Create, update, and delete records through an app API key.
pub const API_KEY_SCOPE_RECORDS_WRITE: &str = "records:write";
/// Download files through an app API key.
pub const API_KEY_SCOPE_FILES_READ: &str = "files:read";
/// Upload and delete files through an app API key.
pub const API_KEY_SCOPE_FILES_WRITE: &str = "files:write";
/// Read collection schemas through an app API key.
pub const API_KEY_SCOPE_SCHEMA_READ: &str = "schema:read";
/// Create, update, and delete collection schemas through an app API key.
pub const API_KEY_SCOPE_SCHEMA_WRITE: &str = "schema:write";
/// Read support tickets through an app API key.
pub const API_KEY_SCOPE_TICKETS_READ: &str = "tickets:read";
/// Create, update, and reply to support tickets through an app API key.
pub const API_KEY_SCOPE_TICKETS_WRITE: &str = "tickets:write";
/// Grant every API-key scope.
pub const API_KEY_SCOPE_ALL: &str = "*";
/// Grant every record scope.
pub const API_KEY_SCOPE_RECORDS_ALL: &str = "records:*";
/// Grant every file scope.
pub const API_KEY_SCOPE_FILES_ALL: &str = "files:*";
/// Grant every collection-schema scope.
pub const API_KEY_SCOPE_SCHEMA_ALL: &str = "schema:*";
/// Grant every support-ticket scope.
pub const API_KEY_SCOPE_TICKETS_ALL: &str = "tickets:*";

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
    pub app_id: String,
    pub name: String,
    pub key_prefix: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub created: DateTime<Utc>,
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Request to create an app API key.
///
/// API-key authorization is deny-by-default. An empty `scopes` list creates a
/// key that authenticates successfully but cannot access scoped data routes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreate {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl ApiKeyCreate {
    /// Create a deny-by-default API-key request.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scopes: Vec::new(),
        }
    }

    /// Grant one explicit operation or domain wildcard scope.
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }
}

/// Newly created API key, including the secret shown only once.
#[derive(Clone, Serialize, Deserialize)]
pub struct ApiKeyCreated {
    #[serde(flatten)]
    pub api_key: ApiKey,
    pub key: String,
}

impl std::fmt::Debug for ApiKeyCreated {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApiKeyCreated")
            .field("api_key", &self.api_key)
            .field("key", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiKeyCreate, ApiKeyCreated, API_KEY_SCOPE_RECORDS_READ};

    #[test]
    fn api_key_request_is_deny_by_default_and_adds_explicit_scopes() {
        let empty = ApiKeyCreate::new("worker");
        assert!(empty.scopes.is_empty());

        let reader = empty.with_scope(API_KEY_SCOPE_RECORDS_READ);
        assert_eq!(reader.scopes, vec!["records:read"]);
    }

    #[test]
    fn created_api_key_requires_the_one_time_secret() {
        let response = serde_json::json!({
            "id": "key-1",
            "app_id": "app-1",
            "name": "worker",
            "key_prefix": "cpd_01234567",
            "scopes": ["records:read"],
            "created": "2026-07-22T00:00:00Z",
            "last_used": null,
            "revoked_at": null
        });

        assert!(serde_json::from_value::<ApiKeyCreated>(response).is_err());
    }

    #[test]
    fn created_api_key_debug_output_redacts_the_secret() {
        let response = serde_json::json!({
            "id": "key-1",
            "app_id": "app-1",
            "name": "worker",
            "key_prefix": "cpd_01234567",
            "scopes": [],
            "created": "2026-07-22T00:00:00Z",
            "last_used": null,
            "revoked_at": null,
            "key": "cpd_this-must-not-appear"
        });
        let created: ApiKeyCreated = serde_json::from_value(response).unwrap();
        let debug = format!("{created:?}");

        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("cpd_this-must-not-appear"));
    }
}
