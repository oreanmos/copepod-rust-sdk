use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{FeatureFlag, ListResult};

impl CopepodClient {
    /// List feature flags for an app.
    pub async fn list_flags(&self, app_id: &str) -> Result<ListResult<FeatureFlag>> {
        self.get(&format!("api/platform/apps/{}/flags", app_id))
            .await
    }

    /// Create a feature flag.
    pub async fn create_flag(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<FeatureFlag> {
        self.post(&format!("api/platform/apps/{}/flags", app_id), body)
            .await
    }

    /// Update a feature flag.
    pub async fn update_flag(
        &self,
        app_id: &str,
        flag_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<FeatureFlag> {
        self.patch(
            &format!("api/platform/apps/{}/flags/{}", app_id, flag_id),
            body,
        )
        .await
    }

    /// Delete a feature flag.
    pub async fn delete_flag(&self, app_id: &str, flag_id: &str) -> Result<()> {
        self.delete(&format!("api/platform/apps/{}/flags/{}", app_id, flag_id))
            .await
    }
}
