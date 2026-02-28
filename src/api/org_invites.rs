use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, OrgInvite};

impl CopepodClient {
    /// List pending invites for an organization.
    pub async fn list_invites(&self, org_id: &str) -> Result<ListResult<OrgInvite>> {
        self.get(&format!("api/platform/orgs/{}/invites", org_id))
            .await
    }

    /// Create a new invite.
    pub async fn create_invite(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<OrgInvite> {
        self.post(&format!("api/platform/orgs/{}/invites", org_id), body)
            .await
    }

    /// Resend an invite.
    pub async fn resend_invite(
        &self,
        org_id: &str,
        invite_id: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!(
                "api/platform/orgs/{}/invites/{}/resend",
                org_id, invite_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Revoke an invite.
    pub async fn revoke_invite(
        &self,
        org_id: &str,
        invite_id: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!(
                "api/platform/orgs/{}/invites/{}/revoke",
                org_id, invite_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Accept an invite using the invite token.
    pub async fn accept_invite(&self, invite_token: &str) -> Result<serde_json::Value> {
        self.post(
            &format!("api/platform/invites/{}/accept", invite_token),
            &serde_json::json!({}),
        )
        .await
    }
}
