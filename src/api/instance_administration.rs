use reqwest::Method;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    InstanceAdministratorsResponse, InstanceRoleResponse, RecoverInstanceOwnerRequest,
    RecoverInstanceOwnerResponse, RevokeInstanceRoleRequest, RevokeInstanceRoleResponse,
    SetInstanceRoleRequest,
};

impl CopepodClient {
    /// List explicit instance-wide administrators.
    pub async fn list_instance_administrators(&self) -> Result<InstanceAdministratorsResponse> {
        self.get("api/platform/instance-administrators").await
    }

    /// Grant or change an explicit instance-wide role.
    pub async fn set_instance_role(
        &self,
        user_id: &str,
        request: &SetInstanceRoleRequest,
    ) -> Result<InstanceRoleResponse> {
        self.put(
            &format!("api/platform/instance-administrators/{user_id}"),
            request,
        )
        .await
    }

    /// Revoke an explicit instance-wide role.
    pub async fn revoke_instance_role(
        &self,
        user_id: &str,
        request: &RevokeInstanceRoleRequest,
    ) -> Result<RevokeInstanceRoleResponse> {
        self.delete_with_body(
            &format!("api/platform/instance-administrators/{user_id}"),
            request,
        )
        .await
    }

    /// Use the orchestrator-provided recovery token to restore an owner.
    ///
    /// This route deliberately does not use session authentication. Rotate or
    /// remove the token immediately after successful incident recovery.
    pub async fn recover_instance_owner(
        &self,
        recovery_token: &str,
        request: &RecoverInstanceOwnerRequest,
    ) -> Result<RecoverInstanceOwnerResponse> {
        let response = self
            .request(Method::POST, "api/platform/auth/admin-recovery")
            .header("x-copepod-admin-recovery-token", recovery_token)
            .json(request)
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }
}
