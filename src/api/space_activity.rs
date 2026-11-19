//! The Space activity log (member of any role): last 90 days, newest first.

use reqwest::Method;

use super::spaces::{base, validate_id};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::SpaceActivityPage;

impl CopepodClient {
    /// List a Space's activity (`GET spaces/{id}/activity`, app-user bearer).
    ///
    /// `limit` and `before` (the previous page's `next_before`) are sent only
    /// when given (the server clamps `limit` to 10-50); `collections` narrows item and comment events and is sent
    /// comma-joined only when not empty. A name holding a comma or a path
    /// character is `InvalidArgument` before any call.
    pub async fn list_space_activity(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        limit: Option<u32>,
        before: Option<&str>,
        collections: &[&str],
    ) -> Result<SpaceActivityPage> {
        validate_id("space_id", space_id)?;
        for name in collections {
            validate_id("collection", name)?;
            if name.contains(',') {
                return Err(CopepodError::InvalidArgument(
                    "collection names must not contain a comma".into(),
                ));
            }
        }
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(before) = before {
            params.push(("before", before.to_owned()));
        }
        if !collections.is_empty() {
            params.push(("collections", collections.join(",")));
        }
        let response = self
            .auth_request(
                Method::GET,
                &format!("{}/spaces/{space_id}/activity", base(org_id, app_id)),
            )
            .await?
            .query(&params)
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }
}
