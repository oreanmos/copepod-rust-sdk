//! Owner-scoped audiences (publishing P8): one newsletter list per app user.
//!
//! The server-facing methods (`put_owner_audience`, `create_owner_subscriber`,
//! the issue endpoints) use the configured API key only; the `me_*` methods
//! use the app-user bearer token only. Coded server errors surface as
//! [`CopepodError::Api`]; branch on `code`: 409 `issue_cancelled`,
//! `issue_not_tested`, `issue_already_sent`, `issue_limit_reached`,
//! `owner_audience_disabled`, 423 `owner_audience_suspended`, 503
//! `owner_audience_upgrading`. Arguments are validated first and rejected with
//! [`CopepodError::InvalidArgument`]. See `docs/owner-audience.md`.

use std::net::IpAddr;

use reqwest::Method;

use super::email::looks_like_email_address;
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    CreateOwnerIssue, CreateOwnerSubscriber, OwnerAccepted, OwnerAudience, OwnerIssue, OwnerPage,
    OwnerSubscriber, OwnerSubscriberStatus, PutOwnerAudience, SubscriberQuery, TestIssueRequest,
};

const MAX_RESOURCE_ID_BYTES: usize = 128;
const MAX_EMAIL_CHARS: usize = 254;
const MAX_DISPLAY_NAME_CHARS: usize = 80;
const MAX_SITE_URL_BYTES: usize = 200;
const MAX_CONSENT_CHARS: usize = 300;
const MAX_SUBJECT_CHARS: usize = 150;
const MAX_ISSUE_BODY_BYTES: usize = 200 * 1024;

fn invalid(message: impl Into<String>) -> CopepodError {
    CopepodError::InvalidArgument(message.into())
}

fn check_consent(text: &str) -> Result<()> {
    if text.trim().is_empty() || text.chars().count() > MAX_CONSENT_CHARS {
        return Err(invalid("consent text must contain 1-300 characters"));
    }
    Ok(())
}

fn check_email(label: &str, email: &str) -> Result<()> {
    let email = email.trim();
    if email.chars().count() > MAX_EMAIL_CHARS || !looks_like_email_address(email) {
        return Err(invalid(format!(
            "{label} must be a valid address of at most 254 characters"
        )));
    }
    Ok(())
}

/// Mirror the server's `invalid_owner_audience` rules.
fn validate_audience(req: &PutOwnerAudience) -> Result<()> {
    if req.display_name.trim().is_empty()
        || req.display_name.chars().count() > MAX_DISPLAY_NAME_CHARS
    {
        return Err(invalid("display name must contain 1-80 characters"));
    }
    if !req.site_url.starts_with("https://") || req.site_url.len() > MAX_SITE_URL_BYTES {
        return Err(invalid(
            "site URL must be an https:// URL of at most 200 bytes",
        ));
    }
    check_consent(&req.consent_text)
}

/// Mirror the server's `invalid_subscriber` rules.
fn validate_subscriber(req: &CreateOwnerSubscriber) -> Result<()> {
    check_email("subscriber email", &req.email)?;
    check_consent(&req.consent_text)?;
    if req.client_ip.trim().parse::<IpAddr>().is_err() {
        return Err(invalid(
            "client_ip must be a single IP address; if you have an X-Forwarded-For list, pass its first hop",
        ));
    }
    Ok(())
}

/// Mirror the server's `invalid_issue` rules (the sanitiser runs server side).
fn validate_issue(req: &CreateOwnerIssue) -> Result<()> {
    if req.subject.trim().is_empty()
        || req.subject.chars().count() > MAX_SUBJECT_CHARS
        || req.subject.contains(['\r', '\n', '\0'])
    {
        return Err(invalid(
            "subject must contain 1-150 characters without line breaks",
        ));
    }
    if req.body_html.trim().is_empty()
        || req.body_html.len() > MAX_ISSUE_BODY_BYTES
        || req.body_html.contains('\0')
    {
        return Err(invalid(
            "body_html must be 1 byte to 200 KiB without null bytes",
        ));
    }
    Ok(())
}

fn validate_query(q: &SubscriberQuery) -> Result<()> {
    if q.status == Some(OwnerSubscriberStatus::Unknown) {
        return Err(invalid("subscriber status filter must be a known status"));
    }
    Ok(())
}

fn validate_resource_id(label: &str, value: &str) -> Result<()> {
    let valid_length = (1..=MAX_RESOURCE_ID_BYTES).contains(&value.len());
    let path_safe = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid_length && path_safe {
        return Ok(());
    }
    Err(CopepodError::InvalidArgument(format!(
        "{label} must contain 1-{MAX_RESOURCE_ID_BYTES} ASCII letters, digits, hyphens, or underscores"
    )))
}

fn validate_app_scope(org_id: &str, app_id: &str) -> Result<()> {
    validate_resource_id("organization ID", org_id)?;
    validate_resource_id("application ID", app_id)
}

fn owner_base(org_id: &str, app_id: &str, owner_user_id: &str) -> Result<String> {
    validate_app_scope(org_id, app_id)?;
    validate_resource_id("owner user ID", owner_user_id)?;
    Ok(format!(
        "api/platform/orgs/{org_id}/apps/{app_id}/owner-audiences/{owner_user_id}"
    ))
}

fn me_base(org_id: &str, app_id: &str, collection: &str) -> Result<String> {
    validate_app_scope(org_id, app_id)?;
    validate_resource_id("collection", collection)?;
    Ok(format!(
        "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/audience"
    ))
}

fn subscriber_query_pairs(q: &SubscriberQuery) -> Vec<(&'static str, String)> {
    let mut pairs = Vec::new();
    if let Some(status) = &q.status {
        pairs.push((
            "status",
            serde_json::to_string(status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
        ));
    }
    if let Some(search) = &q.q {
        pairs.push(("q", search.clone()));
    }
    if let Some(limit) = q.limit {
        pairs.push(("limit", limit.to_string()));
    }
    if let Some(offset) = q.offset {
        pairs.push(("offset", offset.to_string()));
    }
    pairs
}

impl CopepodClient {
    /// Create or update the owner's audience. API key only (scope `owner_audience:manage`).
    pub async fn put_owner_audience(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        req: &PutOwnerAudience,
    ) -> Result<OwnerAudience> {
        let path = owner_base(org_id, app_id, owner_user_id)?;
        validate_audience(req)?;
        let response = self
            .api_key_request(Method::PUT, &path)?
            .json(req)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// The reader sign-up. API key only (scope `owner_audience:write`); the
    /// server answers 202 for every well-formed request.
    pub async fn create_owner_subscriber(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        req: &CreateOwnerSubscriber,
    ) -> Result<OwnerAccepted> {
        let path = format!("{}/subscribers", owner_base(org_id, app_id, owner_user_id)?);
        validate_subscriber(req)?;
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(req)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// Create a draft issue. API key only (scope `owner_audience:send`).
    pub async fn create_owner_issue(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        req: &CreateOwnerIssue,
    ) -> Result<OwnerIssue> {
        let path = format!("{}/issues", owner_base(org_id, app_id, owner_user_id)?);
        validate_issue(req)?;
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(req)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// Send a test of the issue to the owner's verified email. API key only.
    /// The server answers 409 `issue_cancelled` for a cancelled issue, 422
    /// `test_recipient_not_owner` and 429 `test_rate_limited`.
    pub async fn test_owner_issue(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        issue_id: &str,
        to: &str,
    ) -> Result<OwnerAccepted> {
        validate_resource_id("issue ID", issue_id)?;
        check_email("test recipient", to)?;
        let path = format!(
            "{}/issues/{issue_id}/test",
            owner_base(org_id, app_id, owner_user_id)?
        );
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(&TestIssueRequest { to: to.to_string() })
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// Send the issue to the owner's subscribers. API key only; the server
    /// refuses (409 `issue_not_tested`) unless a test covered the current
    /// content, and 409 `issue_cancelled` for a cancelled issue.
    pub async fn send_owner_issue(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        issue_id: &str,
    ) -> Result<OwnerAccepted> {
        validate_resource_id("issue ID", issue_id)?;
        let path = format!(
            "{}/issues/{issue_id}/send",
            owner_base(org_id, app_id, owner_user_id)?
        );
        let response = self.api_key_request(Method::POST, &path)?.send().await?;
        Self::handle_response_pub(response).await
    }

    /// The signed-in owner's own audience. Bearer only.
    pub async fn me_audience(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<OwnerAudience> {
        let path = me_base(org_id, app_id, collection)?;
        let response = self.auth_request(Method::GET, &path).await?.send().await?;
        Self::handle_response_pub(response).await
    }

    /// The signed-in owner's subscribers, filtered and paged. Bearer only.
    pub async fn me_audience_subscribers(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        q: &SubscriberQuery,
    ) -> Result<OwnerPage<OwnerSubscriber>> {
        let path = format!("{}/subscribers", me_base(org_id, app_id, collection)?);
        validate_query(q)?;
        let response = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&subscriber_query_pairs(q))
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// The filtered subscribers as CSV text. Bearer only.
    pub async fn me_audience_subscribers_csv(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        q: &SubscriberQuery,
    ) -> Result<String> {
        let path = format!("{}/subscribers.csv", me_base(org_id, app_id, collection)?);
        validate_query(q)?;
        let response = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&subscriber_query_pairs(q))
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Self::error_from_response(response).await);
        }
        Ok(response.text().await?)
    }

    /// Erase one subscriber (an unsubscribed row leaves only its hash). Bearer only.
    pub async fn me_delete_audience_subscriber(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        id: &str,
    ) -> Result<()> {
        validate_resource_id("subscriber ID", id)?;
        let path = format!("{}/subscribers/{id}", me_base(org_id, app_id, collection)?);
        let response = self
            .auth_request(Method::DELETE, &path)
            .await?
            .send()
            .await?;
        Self::handle_empty_response(response).await
    }

    /// The signed-in owner's issues with delivery counts. Bearer only.
    pub async fn me_audience_issues(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        limit: u32,
        offset: u32,
    ) -> Result<OwnerPage<OwnerIssue>> {
        let path = format!("{}/issues", me_base(org_id, app_id, collection)?);
        let response = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }
}
