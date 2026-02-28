use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AppRole, ListResult, UserRole};

impl CopepodClient {
    /// List roles for an app.
    pub async fn list_roles(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ListResult<AppRole>> {
        self.get(&format!("api/orgs/{}/apps/{}/roles", org_id, app_id))
            .await
    }

    /// Create a new role for an app.
    pub async fn create_role(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<AppRole> {
        self.post(&format!("api/orgs/{}/apps/{}/roles", org_id, app_id), body)
            .await
    }

    /// Update a role.
    pub async fn update_role(
        &self,
        org_id: &str,
        app_id: &str,
        role_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<AppRole> {
        self.patch(
            &format!("api/orgs/{}/apps/{}/roles/{}", org_id, app_id, role_id),
            body,
        )
        .await
    }

    /// Delete a role.
    pub async fn delete_role(
        &self,
        org_id: &str,
        app_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/orgs/{}/apps/{}/roles/{}",
            org_id, app_id, role_id
        ))
        .await
    }

    /// List roles assigned to a user.
    pub async fn list_user_roles(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
    ) -> Result<ListResult<UserRole>> {
        self.get(&format!(
            "api/orgs/{}/apps/{}/users/{}/roles",
            org_id, app_id, user_id
        ))
        .await
    }

    /// Assign a role to a user.
    pub async fn assign_role(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<UserRole> {
        self.post(
            &format!(
                "api/orgs/{}/apps/{}/users/{}/roles",
                org_id, app_id, user_id
            ),
            body,
        )
        .await
    }

    /// Revoke a role from a user.
    pub async fn revoke_role(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/orgs/{}/apps/{}/users/{}/roles/{}",
            org_id, app_id, user_id, role_id
        ))
        .await
    }
}
