//! The Space activity log (member of any role): last 90 days, newest first.

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::SpaceActivityPage;

impl CopepodClient {
    /// List a Space's activity (`GET spaces/{id}/activity`, app-user bearer).
    ///
    /// `limit` and `before` (the previous page's `next_before`) are sent only
    /// when given; `collections` narrows item and comment events and is sent
    /// comma-joined only when not empty. A name holding a comma or a path
    /// character is `InvalidArgument` before any call.
    #[allow(unused_variables)]
    pub async fn list_space_activity(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        limit: Option<u32>,
        before: Option<&str>,
        collections: &[&str],
    ) -> Result<SpaceActivityPage> {
        unimplemented!("toolkit-ui4-ui5 S9")
    }
}
