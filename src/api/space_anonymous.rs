//! Reading a commons without an account, when its owner allows it, and the
//! owner's switch for that.
//!
//! The four `commons/{space_id}` reads send **no credentials at all**. They
//! answer only while the commons' owner has set `anonymous_read` and the
//! app's master switch (`settings.anonymous_commons_enabled`, on by default)
//! is on. A Space that does not exist, is private, is not a commons, or has
//! either switch off answers the same 404 `space_not_found`; each such miss
//! counts against the caller's address (429 `link_rate_limited`), so ids must
//! not be probed. A collection that is not open to every member, or does not
//! exist, is a plain 404 without code. Successful answers may be cached for
//! 30 seconds. `set_space_anonymous_read` needs the owner's app-user bearer
//! token. Contract: copepod `docs/api-integration-reference.md` 5.7
//! ("Anonymous commons reads", d8524de).

use serde_json::Value;

use super::spaces::{base, validate_id};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{CommonsRecordsQuery, ListResult, PublicCommons, PublicFileUrl, Space};

/// Longest search text the server accepts.
const MAX_SEARCH_CHARS: usize = 100;
/// Most comparisons a client filter may hold.
const MAX_FILTER_TERMS: usize = 8;

/// Check a filter the way the anonymous doors do: at most 8 comparisons, no
/// `~` / `!~`. String literals are skipped, so a `~` in a quoted value is
/// fine. A filter that passes can still be refused (hidden fields).
fn validate_filter(filter: &str) -> Result<()> {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut terms = 0usize;
    let mut in_op = false;
    for c in filter.chars() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            in_op = false;
            continue;
        }
        if c == '~' {
            return Err(CopepodError::InvalidArgument(
                "pattern filters (~, !~) are not available on anonymous reads; use q".into(),
            ));
        }
        let op = matches!(c, '=' | '!' | '<' | '>' | '?');
        if op && !in_op {
            terms += 1;
        }
        in_op = op;
    }
    if terms > MAX_FILTER_TERMS {
        return Err(CopepodError::InvalidArgument(format!(
            "a filter on anonymous reads may have at most {MAX_FILTER_TERMS} comparisons"
        )));
    }
    Ok(())
}

fn commons_path(org_id: &str, app_id: &str, space_id: &str) -> String {
    format!("{}/commons/{space_id}", base(org_id, app_id))
}

impl CopepodClient {
    /// Open or close a commons to anonymous readers
    /// (`PATCH spaces/{id} {"anonymous_read": on}`, owner only).
    ///
    /// Errors: 403 `insufficient_role`, 404 `space_not_found`, 400
    /// `not_commons` for a private Space, 409 `anonymous_commons_disabled`
    /// when switching on while the app's master switch is off (see
    /// [`CopepodClient::set_anonymous_commons_enabled`]), 503
    /// `anonymous_commons_upgrading` during a rolling upgrade. Switching off
    /// is never held back and ends access on the next read.
    pub async fn set_space_anonymous_read(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        on: bool,
    ) -> Result<Space> {
        validate_id("space_id", space_id)?;
        self.patch(
            &format!("{}/spaces/{space_id}", base(org_id, app_id)),
            &serde_json::json!({ "anonymous_read": on }),
        )
        .await
    }

    /// The public face of an open commons, without credentials
    /// (`GET commons/{space_id}`): id, name and kind.
    pub async fn get_public_commons(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
    ) -> Result<PublicCommons> {
        validate_id("space_id", space_id)?;
        self.get_public(&commons_path(org_id, app_id, space_id))
            .await
    }

    /// List a collection's records in an open commons, without credentials
    /// (`GET commons/{space_id}/records/{collection}`).
    ///
    /// Only collections whose list and view rules are open to every member
    /// are readable (else plain 404); hidden fields and `created_by` are
    /// stripped. `q` is at most 100 characters; a filter may hold at most 8
    /// comparisons and no `~` / `!~` (checked here first). `per_page` is at
    /// most 100.
    pub async fn list_public_commons_records(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        collection: &str,
        query: &CommonsRecordsQuery,
    ) -> Result<ListResult<Value>> {
        validate_id("space_id", space_id)?;
        validate_id("collection", collection)?;
        if let Some(q) = &query.q {
            // The server trims before it counts.
            if q.trim().chars().count() > MAX_SEARCH_CHARS {
                return Err(CopepodError::InvalidArgument(format!(
                    "q must be at most {MAX_SEARCH_CHARS} characters"
                )));
            }
        }
        if let Some(filter) = &query.filter {
            validate_filter(filter)?;
        }
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(v) = query.q.as_ref().filter(|q| !q.trim().is_empty()) {
            params.push(("q", v.clone()));
        }
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
            "{}/records/{collection}",
            commons_path(org_id, app_id, space_id)
        );
        self.get_public_with_query(&path, &params).await
    }

    /// Get one record of an open commons, without credentials
    /// (`GET commons/{space_id}/records/{collection}/{id}`).
    pub async fn get_public_commons_record(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        collection: &str,
        record_id: &str,
    ) -> Result<Value> {
        validate_id("space_id", space_id)?;
        validate_id("collection", collection)?;
        validate_id("record_id", record_id)?;
        self.get_public(&format!(
            "{}/records/{collection}/{record_id}",
            commons_path(org_id, app_id, space_id)
        ))
        .await
    }

    /// Mint a signed download URL for a file of a record in an open commons,
    /// without credentials
    /// (`GET commons/{space_id}/records/{collection}/{id}/files/{filename}`).
    ///
    /// Only for a file that a visible file field of the record names; any
    /// other case is a plain 404. The returned `url` is a server-relative
    /// path: use [`CopepodClient::download_public_commons_file`] or
    /// [`CopepodClient::resolve_url`]. The link works for about 60 seconds
    /// and answers 401 as soon as the app's master switch is turned off. The
    /// answer is served `no-store`: ask again rather than keeping it.
    pub async fn sign_public_commons_file(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        collection: &str,
        record_id: &str,
        filename: &str,
    ) -> Result<PublicFileUrl> {
        validate_id("space_id", space_id)?;
        validate_id("collection", collection)?;
        validate_id("record_id", record_id)?;
        validate_id("filename", filename)?;
        self.get_public(&format!(
            "{}/records/{collection}/{record_id}/files/{filename}",
            commons_path(org_id, app_id, space_id)
        ))
        .await
    }

    /// Join a server-relative path such as [`PublicFileUrl::url`] onto the
    /// client's base URL. Only paths starting with `/` are accepted.
    pub fn resolve_url(&self, path: &str) -> Result<url::Url> {
        if !path.starts_with('/') || path.starts_with("//") {
            return Err(CopepodError::InvalidArgument(
                "path must be server-relative (start with a single /)".into(),
            ));
        }
        Ok(self.base_url.join(path.trim_start_matches('/'))?)
    }

    /// Download the file a [`PublicFileUrl`] names, without credentials (the
    /// link's signature is the credential; no bearer and no API key is sent).
    ///
    /// Errors: 400 once the link expired, 401 when the signature is not
    /// valid or the app's master switch is off. The body is served
    /// `no-store`.
    pub async fn download_public_commons_file(&self, file: &PublicFileUrl) -> Result<bytes::Bytes> {
        let url = self.resolve_url(&file.url)?;
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        if status.is_success() {
            return Ok(resp.bytes().await?);
        }
        let body = resp.bytes().await.unwrap_or_default();
        let (code, message) = crate::client::decode_error_body(&body);
        Err(CopepodError::Api {
            status: status.as_u16(),
            code,
            message,
        })
    }
}
