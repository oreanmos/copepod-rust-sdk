use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    /// Get cache statistics for an app.
    pub async fn get_cache_stats(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/cache/stats",
            org_id, app_id
        ))
        .await
    }

    /// Flush the entire cache for an app.
    pub async fn flush_cache(&self, org_id: &str, app_id: &str) -> Result<()> {
        self.post_empty(
            &format!("api/platform/orgs/{}/apps/{}/cache/flush", org_id, app_id),
            &serde_json::json!({}),
        )
        .await
    }

    /// Delete a specific cache key for an app.
    pub async fn delete_cache_key(
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
}
