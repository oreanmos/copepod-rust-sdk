use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, Plan};

impl CopepodClient {
    /// List all plans (admin).
    pub async fn list_plans_admin(&self) -> Result<ListResult<Plan>> {
        self.get("api/platform/plans").await
    }

    /// Create a plan (admin).
    pub async fn create_plan(&self, body: &impl serde::Serialize) -> Result<Plan> {
        self.post("api/platform/plans", body).await
    }

    /// Update a plan (admin).
    pub async fn update_plan_admin(
        &self,
        plan_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Plan> {
        self.patch(&format!("api/platform/plans/{}", plan_id), body)
            .await
    }

    /// Delete a plan (admin).
    pub async fn delete_plan(&self, plan_id: &str) -> Result<()> {
        self.delete(&format!("api/platform/plans/{}", plan_id))
            .await
    }
}
