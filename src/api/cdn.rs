use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::CdnRule;

impl CopepodClient {
    /// Get CDN rules for an app.
    pub async fn get_cdn_rules(&self, app_id: &str) -> Result<CdnRule> {
        self.get(&format!("api/platform/apps/{}/cdn", app_id))
            .await
    }

    /// Update CDN rules for an app.
    pub async fn update_cdn_rules(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<CdnRule> {
        self.patch(&format!("api/platform/apps/{}/cdn", app_id), body)
            .await
    }

    /// Purge CDN cache for an app.
    pub async fn purge_cdn(&self, app_id: &str) -> Result<serde_json::Value> {
        self.post(
            &format!("api/platform/apps/{}/cdn/purge", app_id),
            &serde_json::json!({}),
        )
        .await
    }
}
