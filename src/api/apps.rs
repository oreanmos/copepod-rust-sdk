use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ApiKey, App, ListResult};

impl CopepodClient {
    /// List all apps in an organization.
    pub async fn list_apps(&self, org_id: &str) -> Result<ListResult<App>> {
        self.get(&format!("api/orgs/{}/apps", org_id)).await
    }

    /// Get an app by ID.
    pub async fn get_app(&self, org_id: &str, app_id: &str) -> Result<App> {
        self.get(&format!("api/orgs/{}/apps/{}", org_id, app_id))
            .await
    }

    /// Create a new app.
    pub async fn create_app(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<App> {
        self.post(&format!("api/orgs/{}/apps", org_id), body).await
    }

    /// Update an app.
    pub async fn update_app(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<App> {
        self.patch(&format!("api/orgs/{}/apps/{}", org_id, app_id), body)
            .await
    }

    /// Delete an app.
    pub async fn delete_app(&self, org_id: &str, app_id: &str) -> Result<()> {
        self.delete(&format!("api/orgs/{}/apps/{}", org_id, app_id))
            .await
    }

    /// List API keys for an app.
    pub async fn list_api_keys(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ListResult<ApiKey>> {
        self.get(&format!("api/orgs/{}/apps/{}/api-keys", org_id, app_id))
            .await
    }

    /// Create a new API key for an app.
    pub async fn create_api_key(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<ApiKey> {
        self.post(
            &format!("api/orgs/{}/apps/{}/api-keys", org_id, app_id),
            body,
        )
        .await
    }

    /// Revoke an API key.
    pub async fn revoke_api_key(
        &self,
        org_id: &str,
        app_id: &str,
        key_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/orgs/{}/apps/{}/api-keys/{}",
            org_id, app_id, key_id
        ))
        .await
    }
}
