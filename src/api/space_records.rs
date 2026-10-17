//! Record, file and realtime calls on space-scoped collections: the same
//! routes as the app-level ones plus `X-Space-Id` (or `?space_id=` for
//! realtime). A missing header is 400 `space_required`; a non-member gets 404
//! `space_not_found`; a read-only role gets 403 `insufficient_role`.

use bytes::Bytes;
use futures_util::stream::Stream;
use reqwest::header::AUTHORIZATION;
use reqwest::Method;
use serde_json::Value;

use super::spaces::validate_id;
use crate::client::{decode_error_body, CopepodClient};
use crate::error::{CopepodError, Result};
use crate::models::{RealtimeSubscriptionOptions, RecordEvent, RecordUpdateOptions};

/// Header naming the Space of a record or file call.
pub const SPACE_ID_HEADER: &str = "X-Space-Id";

fn records_path(org_id: &str, app_id: &str, collection: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/records/{collection}")
}

fn files_path(org_id: &str, app_id: &str, collection: &str, record_id: &str, name: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/files/{collection}/{record_id}/{name}")
}

impl CopepodClient {
    /// Create a record in a space-scoped collection.
    pub async fn create_record_in_space(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        space_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        validate_id("space_id", space_id)?;
        let resp = self
            .auth_request(Method::POST, &records_path(org_id, app_id, collection))
            .await?
            .header(SPACE_ID_HEADER, space_id)
            .json(body)
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Update a record, optionally inside a Space and/or guarded by `If-Match`.
    ///
    /// With [`RecordUpdateOptions::if_match`] the server applies the update only
    /// when the stored revision equals it and bumps the revision itself;
    /// otherwise it answers 409 `revision_conflict`
    /// ([`CopepodError::revision_conflict_details`] returns `current_rev`).
    /// `If-Match` on a collection without `revision_field` is a 400. Send it
    /// only to servers that support it: an older leader ignores the header.
    pub async fn update_record_with(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        record_id: &str,
        body: &impl serde::Serialize,
        options: &RecordUpdateOptions,
    ) -> Result<Value> {
        if let Some(space_id) = &options.space_id {
            validate_id("space_id", space_id)?;
        }
        if let Some(rev) = &options.if_match {
            if rev.trim().is_empty() || rev.chars().any(char::is_control) {
                return Err(CopepodError::InvalidArgument(
                    "if_match must be a non-empty revision".into(),
                ));
            }
        }
        let path = format!("{}/{record_id}", records_path(org_id, app_id, collection));
        let mut request = self.auth_request(Method::PATCH, &path).await?;
        if let Some(space_id) = &options.space_id {
            request = request.header(SPACE_ID_HEADER, space_id);
        }
        if let Some(rev) = &options.if_match {
            request = request.header(reqwest::header::IF_MATCH, rev);
        }
        let resp = request.json(body).send().await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Delete a record of a space-scoped collection.
    pub async fn delete_record_in_space(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        space_id: &str,
        record_id: &str,
    ) -> Result<()> {
        validate_id("space_id", space_id)?;
        let path = format!("{}/{record_id}", records_path(org_id, app_id, collection));
        let resp = self
            .auth_request(Method::DELETE, &path)
            .await?
            .header(SPACE_ID_HEADER, space_id)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(api_error(resp).await)
        }
    }

    /// Upload a file of a space-scoped record. Needs a writing role; the
    /// bytes count against the Space owner's `storage_gb` quota
    /// (422 `storage_quota_exceeded`).
    #[allow(clippy::too_many_arguments)]
    pub async fn upload_file_in_space(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        space_id: &str,
        record_id: &str,
        data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<Value> {
        validate_id("space_id", space_id)?;
        let path = files_path(org_id, app_id, collection, record_id, filename);
        self.ensure_auth().await?;
        let mut builder = self
            .request(Method::POST, &path)
            .header(SPACE_ID_HEADER, space_id)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .body(data);
        if let Some(pair) = self.token_store.get().await {
            builder = builder.header(AUTHORIZATION, format!("Bearer {}", pair.token));
        }
        let resp = builder.send().await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Download a file of a space-scoped record (any member).
    pub async fn download_file_in_space(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        space_id: &str,
        record_id: &str,
        filename: &str,
    ) -> Result<Bytes> {
        validate_id("space_id", space_id)?;
        let path = files_path(org_id, app_id, collection, record_id, filename);
        let resp = self
            .auth_request(Method::GET, &path)
            .await?
            .header(SPACE_ID_HEADER, space_id)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.bytes().await?)
        } else {
            Err(api_error(resp).await)
        }
    }

    /// Delete a file of a space-scoped record (needs a writing role).
    pub async fn delete_file_in_space(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        space_id: &str,
        record_id: &str,
        filename: &str,
    ) -> Result<()> {
        validate_id("space_id", space_id)?;
        let path = files_path(org_id, app_id, collection, record_id, filename);
        let resp = self
            .auth_request(Method::DELETE, &path)
            .await?
            .header(SPACE_ID_HEADER, space_id)
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(api_error(resp).await)
        }
    }

    /// Subscribe to the realtime events of one Space (`?space_id=`). Events
    /// carry [`RecordEvent::space_id`]; a non-member gets 404
    /// `space_not_found`, and a removed member's stream ends within a minute.
    /// The plain [`CopepodClient::subscribe`] never receives Space events.
    pub async fn subscribe_space(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        options: RealtimeSubscriptionOptions,
    ) -> Result<impl Stream<Item = Result<RecordEvent>> + Unpin> {
        validate_id("space_id", space_id)?;
        self.subscribe_scoped(org_id, app_id, Some(space_id), options)
            .await
    }
}

/// Map a non-success response to the SDK's API error.
pub(crate) async fn api_error(resp: reqwest::Response) -> CopepodError {
    let status = resp.status().as_u16();
    let bytes = resp.bytes().await.unwrap_or_default();
    let (code, message) = decode_error_body(&bytes);
    CopepodError::Api {
        status,
        code,
        message,
    }
}
