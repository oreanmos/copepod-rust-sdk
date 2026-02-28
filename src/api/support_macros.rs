use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, SupportMacro};

impl CopepodClient {
    /// List support macros for an organization.
    pub async fn list_macros(&self, org_id: &str) -> Result<ListResult<SupportMacro>> {
        self.get(&format!(
            "api/platform/orgs/{}/support/macros",
            org_id
        ))
        .await
    }

    /// Create a support macro.
    pub async fn create_macro(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<SupportMacro> {
        self.post(
            &format!("api/platform/orgs/{}/support/macros", org_id),
            body,
        )
        .await
    }

    /// Update a support macro.
    pub async fn update_macro(
        &self,
        org_id: &str,
        macro_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<SupportMacro> {
        self.patch(
            &format!(
                "api/platform/orgs/{}/support/macros/{}",
                org_id, macro_id
            ),
            body,
        )
        .await
    }

    /// Delete a support macro.
    pub async fn delete_macro(&self, org_id: &str, macro_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/support/macros/{}",
            org_id, macro_id
        ))
        .await
    }
}
