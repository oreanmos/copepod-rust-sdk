//! The signed-in app user's own display name.

use super::spaces::{base, validate_id};
use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::AppProfile;

impl CopepodClient {
    /// Read the caller's name (`GET auth/{collection}/me/profile`, app-user bearer).
    pub async fn get_my_profile(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<AppProfile> {
        validate_id("collection", collection)?;
        self.get(&profile_path(org_id, app_id, collection)).await
    }

    /// Change the caller's name (`PATCH auth/{collection}/me/profile`); `None`
    /// clears it. 422 `name_invalid` for a bad name.
    pub async fn update_my_profile(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        name: Option<&str>,
    ) -> Result<AppProfile> {
        validate_id("collection", collection)?;
        self.patch(
            &profile_path(org_id, app_id, collection),
            &serde_json::json!({ "name": name }),
        )
        .await
    }
}

fn profile_path(org_id: &str, app_id: &str, collection: &str) -> String {
    format!("{}/auth/{collection}/me/profile", base(org_id, app_id))
}
