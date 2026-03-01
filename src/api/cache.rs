use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    /// List cache keys for an app.
    pub async fn list_cache_keys(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/cache",
            org_id, app_id
        ))
        .await
    }

    /// Get a specific cache entry by key.
    pub async fn get_cache_entry(
        &self,
        org_id: &str,
        app_id: &str,
        key: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/cache/{}",
            org_id, app_id, key
        ))
        .await
    }

    /// Set a cache entry by key.
    pub async fn set_cache_entry(
        &self,
        org_id: &str,
        app_id: &str,
        key: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!("api/platform/orgs/{}/apps/{}/cache/{}", org_id, app_id, key),
            body,
        )
        .await
    }

    /// Delete a specific cache entry by key.
    pub async fn delete_cache_entry(
        &self,
        org_id: &str,
        app_id: &str,
        key: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/cache/{}",
            org_id, app_id, key
        ))
        .await
    }

    /// Flush the entire cache for an app.
    pub async fn flush_cache(&self, org_id: &str, app_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/cache",
            org_id, app_id
        ))
        .await
    }

    /// Get global cache statistics.
    pub async fn get_cache_stats(&self) -> Result<Value> {
        self.get("api/platform/cache/stats").await
    }
}
