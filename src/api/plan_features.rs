use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ItemsResponse, PlanFeatureMapping, PlanFeatureMappingInput};

impl CopepodClient {
    /// Set feature mappings for a plan (replaces existing mappings).
    pub async fn set_plan_features(
        &self,
        plan_id: &str,
        mappings: &[PlanFeatureMappingInput],
    ) -> Result<ItemsResponse<PlanFeatureMapping>> {
        self.put(
            &format!("api/platform/plans/{}/features", plan_id),
            &mappings,
        )
        .await
    }

    /// List feature mappings for a plan.
    pub async fn list_plan_features(
        &self,
        plan_id: &str,
    ) -> Result<ItemsResponse<PlanFeatureMapping>> {
        self.get(&format!("api/platform/plans/{}/features", plan_id))
            .await
    }
}
