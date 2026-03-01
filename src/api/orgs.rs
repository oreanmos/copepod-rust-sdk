use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, Org, OrgMember};

impl CopepodClient {
    /// List all organizations.
    pub async fn list_orgs(&self) -> Result<ListResult<Org>> {
        self.get("api/platform/orgs").await
    }

    /// Get an organization by ID.
    pub async fn get_org(&self, id: &str) -> Result<Org> {
        self.get(&format!("api/platform/orgs/{}", id)).await
    }

    /// Create a new organization.
    pub async fn create_org(&self, body: &impl serde::Serialize) -> Result<Org> {
        self.post("api/platform/orgs", body).await
    }

    /// Update an organization.
    pub async fn update_org(&self, id: &str, body: &impl serde::Serialize) -> Result<Org> {
        self.patch(&format!("api/platform/orgs/{}", id), body).await
    }

    /// Delete an organization.
    pub async fn delete_org(&self, id: &str) -> Result<()> {
        self.delete(&format!("api/platform/orgs/{}", id)).await
    }

    /// List members of an organization.
    pub async fn list_org_members(&self, org_id: &str) -> Result<ListResult<OrgMember>> {
        self.get(&format!("api/platform/orgs/{}/members", org_id)).await
    }

    /// Add a member to an organization.
    pub async fn add_org_member(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<OrgMember> {
        self.post(&format!("api/platform/orgs/{}/members", org_id), body).await
    }

    /// Update an organization member's role.
    pub async fn update_org_member(
        &self,
        org_id: &str,
        user_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<OrgMember> {
        self.patch(&format!("api/platform/orgs/{}/members/{}", org_id, user_id), body)
            .await
    }

    /// Remove a member from an organization.
    pub async fn remove_org_member(&self, org_id: &str, user_id: &str) -> Result<()> {
        self.delete(&format!("api/platform/orgs/{}/members/{}", org_id, user_id))
            .await
    }
}
