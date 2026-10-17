//! Spaces: shared data containers, their members and invitations.
//!
//! All routes need an **app-user bearer token**; API keys and dashboard
//! tokens get 401. A Space the caller does not belong to is 404
//! `space_not_found`, a role that cannot do the operation is 403
//! `insufficient_role`.

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    ItemsResponse, Space, SpaceCreate, SpaceInvite, SpaceInviteCreate, SpaceKind, SpaceMember,
    SpaceRole, SpaceWithRole,
};

/// Reject an empty id or one that is not a single URL path segment / header
/// value, before any network call.
pub(crate) fn validate_id(what: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value.chars().any(|c| {
            c == '/' || c == '?' || c == '#' || c == '%' || c.is_whitespace() || c.is_control()
        })
    {
        return Err(CopepodError::InvalidArgument(format!(
            "{what} must be a non-empty id without slashes or whitespace"
        )));
    }
    Ok(())
}

fn base(org_id: &str, app_id: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}")
}

impl CopepodClient {
    /// Create a Space; the caller becomes its owner (`POST spaces`).
    ///
    /// Only [`SpaceKind::Private`] can be created for now. Errors: 400 for an
    /// invalid name or kind, 503 `spaces_upgrading` during a rolling upgrade.
    pub async fn create_space(
        &self,
        org_id: &str,
        app_id: &str,
        body: &SpaceCreate,
    ) -> Result<Space> {
        if body.name.trim().is_empty() {
            return Err(CopepodError::InvalidArgument(
                "space name must not be empty".into(),
            ));
        }
        if body.kind != SpaceKind::Private {
            return Err(CopepodError::InvalidArgument(
                "only private spaces can be created".into(),
            ));
        }
        self.post(&format!("{}/spaces", base(org_id, app_id)), body)
            .await
    }

    /// List the caller's live Spaces with the caller's role in each.
    pub async fn list_spaces(&self, org_id: &str, app_id: &str) -> Result<Vec<SpaceWithRole>> {
        let list: ItemsResponse<SpaceWithRole> = self
            .get(&format!("{}/spaces", base(org_id, app_id)))
            .await?;
        Ok(list.items)
    }

    /// Fetch one Space with the caller's role. 404 `space_not_found` for a
    /// non-member.
    pub async fn get_space(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
    ) -> Result<SpaceWithRole> {
        validate_id("space_id", space_id)?;
        self.get(&format!("{}/spaces/{space_id}", base(org_id, app_id)))
            .await
    }

    /// Rename a Space (owner only).
    pub async fn rename_space(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        name: &str,
    ) -> Result<Space> {
        validate_id("space_id", space_id)?;
        if name.trim().is_empty() {
            return Err(CopepodError::InvalidArgument(
                "space name must not be empty".into(),
            ));
        }
        self.patch(
            &format!("{}/spaces/{space_id}", base(org_id, app_id)),
            &serde_json::json!({ "name": name }),
        )
        .await
    }

    /// Delete a Space (owner only). It stops resolving at once.
    pub async fn delete_space(&self, org_id: &str, app_id: &str, space_id: &str) -> Result<()> {
        validate_id("space_id", space_id)?;
        self.delete(&format!("{}/spaces/{space_id}", base(org_id, app_id)))
            .await
    }

    /// List the members of a Space (any member).
    pub async fn list_space_members(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
    ) -> Result<Vec<SpaceMember>> {
        validate_id("space_id", space_id)?;
        let list: ItemsResponse<SpaceMember> = self
            .get(&format!(
                "{}/spaces/{space_id}/members",
                base(org_id, app_id)
            ))
            .await?;
        Ok(list.items)
    }

    /// Change a member's role (owner only). A Space always keeps one owner:
    /// 409 `last_owner`.
    pub async fn set_space_member_role(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        user_id: &str,
        role: SpaceRole,
    ) -> Result<SpaceMember> {
        validate_id("space_id", space_id)?;
        validate_id("user_id", user_id)?;
        self.patch(
            &format!(
                "{}/spaces/{space_id}/members/{user_id}",
                base(org_id, app_id)
            ),
            &serde_json::json!({ "role": role }),
        )
        .await
    }

    /// Remove a member (owner), or leave a Space by passing the caller's own
    /// user id. 409 `last_owner` for the last owner.
    pub async fn remove_space_member(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        user_id: &str,
    ) -> Result<()> {
        validate_id("space_id", space_id)?;
        validate_id("user_id", user_id)?;
        self.delete(&format!(
            "{}/spaces/{space_id}/members/{user_id}",
            base(org_id, app_id)
        ))
        .await
    }

    /// Invite an email address to a Space (owner only, verified email).
    ///
    /// The role is `owner`, `editor` or `viewer`. The token is only in the
    /// email. Errors: 403 `email_not_verified`, 429 `invite_rate_limited`,
    /// 503 when the platform cannot send email.
    pub async fn create_space_invite(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        body: &SpaceInviteCreate,
    ) -> Result<SpaceInvite> {
        validate_id("space_id", space_id)?;
        if body.email.trim().is_empty() {
            return Err(CopepodError::InvalidArgument(
                "invite email must not be empty".into(),
            ));
        }
        if !matches!(
            body.role,
            SpaceRole::Owner | SpaceRole::Editor | SpaceRole::Viewer
        ) {
            return Err(CopepodError::InvalidArgument(
                "invite role must be owner, editor or viewer".into(),
            ));
        }
        self.post(
            &format!("{}/spaces/{space_id}/invites", base(org_id, app_id)),
            body,
        )
        .await
    }

    /// List the pending, unexpired invites of a Space (owner only).
    pub async fn list_space_invites(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
    ) -> Result<Vec<SpaceInvite>> {
        validate_id("space_id", space_id)?;
        let list: ItemsResponse<SpaceInvite> = self
            .get(&format!(
                "{}/spaces/{space_id}/invites",
                base(org_id, app_id)
            ))
            .await?;
        Ok(list.items)
    }

    /// Revoke a pending invite (owner only).
    pub async fn revoke_space_invite(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        invite_id: &str,
    ) -> Result<()> {
        validate_id("space_id", space_id)?;
        validate_id("invite_id", invite_id)?;
        self.delete(&format!(
            "{}/spaces/{space_id}/invites/{invite_id}",
            base(org_id, app_id)
        ))
        .await
    }

    /// Accept an invite with the emailed token. The signed-in user's
    /// verified email must match the invite. Returns the Space with the
    /// caller's role. Errors: 403 `email_not_verified` /
    /// `invite_email_mismatch`, 410 `invite_expired`, 404 `invite_not_found`,
    /// 409 `already_member`.
    pub async fn accept_space_invite(
        &self,
        org_id: &str,
        app_id: &str,
        token: &str,
    ) -> Result<SpaceWithRole> {
        if token.trim().is_empty() {
            return Err(CopepodError::InvalidArgument(
                "invite token must not be empty".into(),
            ));
        }
        self.post(
            &format!("{}/space-invites/accept", base(org_id, app_id)),
            &serde_json::json!({ "token": token }),
        )
        .await
    }
}
