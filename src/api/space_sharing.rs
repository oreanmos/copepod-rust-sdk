//! Share links: an owner or editor opens a Space, or one record of it, to
//! anyone holding the link's token. Management routes need an app-user
//! bearer token; the `shared/{token}` reads send **no credentials at all**
//! (the token in the path is the only one) and grant reading, never
//! membership.
//!
//! Errors on management: 404 `space_not_found` / `link_not_found`, 403
//! `insufficient_role`, 400 for a bad scope or expiry, 409 `share_link_limit`,
//! 429 `share_link_rate_limited`, 503 `share_links_upgrading`. Errors on a
//! shared read: 404 `link_not_found`, 410 `link_revoked` / `link_expired`,
//! 429 `link_rate_limited` (failed lookups from one address).

use reqwest::Method;
use serde_json::Value;

use super::spaces::{base, validate_id};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    ItemsResponse, ListResult, ShareLinkCreate, ShareLinkIssued, ShareLinkScope,
    SharedRecordsQuery, SpaceShareLink,
};

/// Longest token the server looks up.
const MAX_TOKEN_BYTES: usize = 256;

fn validate_expiry(expires_at: Option<&str>) -> Result<()> {
    if let Some(value) = expires_at {
        if chrono::DateTime::parse_from_rfc3339(value).is_err() {
            return Err(CopepodError::InvalidArgument(
                "expires_at must be an RFC 3339 timestamp".into(),
            ));
        }
    }
    Ok(())
}

fn validate_token(token: &str) -> Result<()> {
    validate_id("token", token)?;
    if token.len() > MAX_TOKEN_BYTES {
        return Err(CopepodError::InvalidArgument("token is too long".into()));
    }
    Ok(())
}

impl CopepodClient {
    /// Make a share link (`POST spaces/{id}/share-links`, owner or editor).
    ///
    /// The token is returned once, in [`ShareLinkIssued`]. A record scope
    /// must name a record the caller can view in a space-scoped collection.
    pub async fn create_share_link(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        body: &ShareLinkCreate,
    ) -> Result<ShareLinkIssued> {
        validate_id("space_id", space_id)?;
        validate_expiry(body.expires_at.as_deref())?;
        if let ShareLinkScope::Record {
            record_collection,
            record_id,
        } = &body.scope
        {
            validate_id("record_collection", record_collection)?;
            validate_id("record_id", record_id)?;
        }
        self.post(
            &format!("{}/spaces/{space_id}/share-links", base(org_id, app_id)),
            body,
        )
        .await
    }

    /// List a Space's unrevoked share links, never their tokens (owner or
    /// editor).
    pub async fn list_share_links(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
    ) -> Result<Vec<SpaceShareLink>> {
        validate_id("space_id", space_id)?;
        let list: ItemsResponse<SpaceShareLink> = self
            .get(&format!(
                "{}/spaces/{space_id}/share-links",
                base(org_id, app_id)
            ))
            .await?;
        Ok(list.items)
    }

    /// Revoke a link (`DELETE share-links/{id}`, owner or editor). Its token
    /// answers 410 `link_revoked` from then on.
    pub async fn revoke_share_link(&self, org_id: &str, app_id: &str, link_id: &str) -> Result<()> {
        validate_id("link_id", link_id)?;
        self.delete(&format!("{}/share-links/{link_id}", base(org_id, app_id)))
            .await
    }

    /// Revoke a link and issue a new token with the same scope and
    /// permission (`POST share-links/{id}/rotate`, owner or editor).
    ///
    /// `expires_at` defaults to the old expiry while it lies ahead. The new
    /// link belongs to the caller, and a record scope is checked again as at
    /// creation. A link already rotated or revoked is 410 `link_revoked`.
    pub async fn rotate_share_link(
        &self,
        org_id: &str,
        app_id: &str,
        link_id: &str,
        expires_at: Option<&str>,
    ) -> Result<ShareLinkIssued> {
        validate_id("link_id", link_id)?;
        validate_expiry(expires_at)?;
        self.post(
            &format!("{}/share-links/{link_id}/rotate", base(org_id, app_id)),
            &serde_json::json!({ "expires_at": expires_at }),
        )
        .await
    }

    /// List records through a link, without any credentials
    /// (`GET shared/{token}/records/{collection}`).
    ///
    /// Hidden fields and `created_by` are stripped; a record link returns
    /// only its record. Filter, sort and `fields` may name only the
    /// collection's visible fields (else 400); `per_page` is at most 100.
    pub async fn list_shared_records(
        &self,
        org_id: &str,
        app_id: &str,
        token: &str,
        collection: &str,
        query: &SharedRecordsQuery,
    ) -> Result<ListResult<Value>> {
        validate_token(token)?;
        validate_id("collection", collection)?;
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(v) = &query.filter {
            params.push(("filter", v.clone()));
        }
        if let Some(v) = &query.sort {
            params.push(("sort", v.clone()));
        }
        if let Some(v) = &query.fields {
            params.push(("fields", v.clone()));
        }
        if let Some(v) = query.page {
            params.push(("page", v.to_string()));
        }
        if let Some(v) = query.per_page {
            params.push(("per_page", v.to_string()));
        }
        let path = format!(
            "{}/shared/{token}/records/{collection}",
            base(org_id, app_id)
        );
        let resp = self
            .request(Method::GET, &path)
            .query(&params)
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Get one record through a link, without any credentials
    /// (`GET shared/{token}/records/{collection}/{id}`). A record link
    /// answers 404 for any other record.
    pub async fn get_shared_record(
        &self,
        org_id: &str,
        app_id: &str,
        token: &str,
        collection: &str,
        record_id: &str,
    ) -> Result<Value> {
        validate_token(token)?;
        validate_id("collection", collection)?;
        validate_id("record_id", record_id)?;
        self.get_public(&format!(
            "{}/shared/{token}/records/{collection}/{record_id}",
            base(org_id, app_id)
        ))
        .await
    }
}
