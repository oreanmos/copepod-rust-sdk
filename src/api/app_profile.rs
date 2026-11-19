//! The signed-in app user's own display name.

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::AppProfile;

impl CopepodClient {
    /// Read the caller's name (`GET auth/{collection}/me/profile`, app-user bearer).
    #[allow(unused_variables)]
    pub async fn get_my_profile(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<AppProfile> {
        unimplemented!("toolkit-ui4-ui5 S9")
    }

    /// Change the caller's name (`PATCH auth/{collection}/me/profile`); `None`
    /// clears it. 422 `name_invalid` for a bad name.
    #[allow(unused_variables)]
    pub async fn update_my_profile(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        name: Option<&str>,
    ) -> Result<AppProfile> {
        unimplemented!("toolkit-ui4-ui5 S9")
    }
}
