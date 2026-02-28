use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{Entitlement, ListResult};

impl CopepodClient {
    /// Get entitlements for an organization.
    pub async fn get_entitlements(
        &self,
        org_id: &str,
    ) -> Result<ListResult<Entitlement>> {
        self.get(&format!(
            "api/platform/orgs/{}/entitlements",
            org_id
        ))
        .await
    }

    /// Create an entitlement override.
    pub async fn create_entitlement_override(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Entitlement> {
        self.post(
            &format!(
                "api/platform/orgs/{}/entitlements/overrides",
                org_id
            ),
            body,
        )
        .await
    }

    /// Delete an entitlement override.
    pub async fn delete_entitlement_override(
        &self,
        org_id: &str,
        override_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/entitlements/overrides/{}",
            org_id, override_id
        ))
        .await
    }

    /// Resolve entitlements (check current effective entitlements).
    pub async fn resolve_entitlements(&self) -> Result<serde_json::Value> {
        self.get("api/platform/entitlements/resolve").await
    }
}
