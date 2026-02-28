use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{IamRole, ListResult, PolicyBinding, ServiceAccount};

impl CopepodClient {
    // --- IAM Roles ---

    /// List IAM roles for an organization.
    pub async fn list_iam_roles(
        &self,
        org_id: &str,
    ) -> Result<ListResult<IamRole>> {
        self.get(&format!("api/platform/orgs/{}/roles", org_id))
            .await
    }

    /// Create an IAM role.
    pub async fn create_iam_role(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<IamRole> {
        self.post(&format!("api/platform/orgs/{}/roles", org_id), body)
            .await
    }

    /// Get an IAM role.
    pub async fn get_iam_role(
        &self,
        org_id: &str,
        role_id: &str,
    ) -> Result<IamRole> {
        self.get(&format!(
            "api/platform/orgs/{}/roles/{}",
            org_id, role_id
        ))
        .await
    }

    /// Update an IAM role.
    pub async fn update_iam_role(
        &self,
        org_id: &str,
        role_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<IamRole> {
        self.patch(
            &format!("api/platform/orgs/{}/roles/{}", org_id, role_id),
            body,
        )
        .await
    }

    /// Delete an IAM role.
    pub async fn delete_iam_role(
        &self,
        org_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/roles/{}",
            org_id, role_id
        ))
        .await
    }

    // --- Policy Bindings ---

    /// List policy bindings for an organization.
    pub async fn list_policy_bindings(
        &self,
        org_id: &str,
    ) -> Result<ListResult<PolicyBinding>> {
        self.get(&format!(
            "api/platform/orgs/{}/policy-bindings",
            org_id
        ))
        .await
    }

    /// Create a policy binding.
    pub async fn create_policy_binding(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<PolicyBinding> {
        self.post(
            &format!("api/platform/orgs/{}/policy-bindings", org_id),
            body,
        )
        .await
    }

    /// Delete a policy binding.
    pub async fn delete_policy_binding(
        &self,
        org_id: &str,
        binding_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/policy-bindings/{}",
            org_id, binding_id
        ))
        .await
    }

    // --- Service Accounts ---

    /// List service accounts for an organization.
    pub async fn list_service_accounts(
        &self,
        org_id: &str,
    ) -> Result<ListResult<ServiceAccount>> {
        self.get(&format!(
            "api/platform/orgs/{}/service-accounts",
            org_id
        ))
        .await
    }

    /// Create a service account.
    pub async fn create_service_account(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<ServiceAccount> {
        self.post(
            &format!("api/platform/orgs/{}/service-accounts", org_id),
            body,
        )
        .await
    }

    /// Delete a service account.
    pub async fn delete_service_account(
        &self,
        org_id: &str,
        sa_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/service-accounts/{}",
            org_id, sa_id
        ))
        .await
    }
}
