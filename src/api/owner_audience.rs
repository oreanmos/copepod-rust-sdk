//! Owner-scoped audiences (publishing P8): one newsletter list per app user.
//!
//! The server-facing methods (`put_owner_audience`, `create_owner_subscriber`,
//! the issue endpoints) use the configured API key only; the `me_*` methods
//! use the app-user bearer token only. Coded server errors surface as
//! [`CopepodError::Api`].

use reqwest::Method;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    Accepted, CreateOwnerIssue, CreateOwnerSubscriber, OwnerAudience, OwnerIssue, OwnerSubscriber,
    Page, PutOwnerAudience, SubscriberQuery, TestIssueRequest,
};

const MAX_RESOURCE_ID_BYTES: usize = 128;

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
    ) -> Result<Accepted> {
        let path = format!("{}/subscribers", owner_base(org_id, app_id, owner_user_id)?);
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
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(req)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// Send a test of the issue to the owner's verified email. API key only.
    pub async fn test_owner_issue(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        issue_id: &str,
        to: &str,
    ) -> Result<Accepted> {
        validate_resource_id("issue ID", issue_id)?;
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
    /// refuses (409 `issue_not_tested`) unless a test covered the current content.
    pub async fn send_owner_issue(
        &self,
        org_id: &str,
        app_id: &str,
        owner_user_id: &str,
        issue_id: &str,
    ) -> Result<Accepted> {
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
    ) -> Result<Page<OwnerSubscriber>> {
        let path = format!("{}/subscribers", me_base(org_id, app_id, collection)?);
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
    ) -> Result<Page<OwnerIssue>> {
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
