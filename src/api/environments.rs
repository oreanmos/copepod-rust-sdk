use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AppEnv, ConfigEntry, ListResult};

impl CopepodClient {
    /// List environments for an app.
    pub async fn list_envs(&self, app_id: &str) -> Result<ListResult<AppEnv>> {
        self.get(&format!("api/platform/apps/{}/envs", app_id))
            .await
    }

    /// Create a new environment.
    pub async fn create_env(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<AppEnv> {
        self.post(&format!("api/platform/apps/{}/envs", app_id), body)
            .await
    }

    /// Delete an environment.
    pub async fn delete_env(&self, app_id: &str, env_id: &str) -> Result<()> {
        self.delete(&format!("api/platform/apps/{}/envs/{}", app_id, env_id))
            .await
    }

    /// Get remote config entries for an app.
    pub async fn get_config(&self, app_id: &str) -> Result<ListResult<ConfigEntry>> {
        self.get(&format!("api/platform/apps/{}/config", app_id))
            .await
    }

    /// Set a config entry.
    pub async fn set_config(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<ConfigEntry> {
        self.put(&format!("api/platform/apps/{}/config", app_id), body)
            .await
    }

    /// Delete config entries for an app.
    pub async fn delete_config(&self, app_id: &str) -> Result<()> {
        self.delete(&format!("api/platform/apps/{}/config", app_id))
            .await
    }
}
