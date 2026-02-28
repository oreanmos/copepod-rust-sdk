use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    /// Get usage analytics for an organization.
    pub async fn get_usage_analytics(
        &self,
        org_id: &str,
    ) -> Result<serde_json::Value> {
        self.get(&format!(
            "api/platform/orgs/{}/usage/analytics",
            org_id
        ))
        .await
    }
}
