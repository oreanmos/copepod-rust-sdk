use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    Accepted, CreateOwnerIssue, CreateOwnerSubscriber, OwnerAudience, OwnerIssue, OwnerSubscriber,
    Page, PutOwnerAudience, SubscriberQuery,
};

/// Owner-audience helpers bound to an organization and application.
#[derive(Debug, Clone)]
pub struct ScopedOwnerAudienceClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedOwnerAudienceClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: &str, app_id: &str) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// Create or update an owner's audience (API key).
    pub async fn put(&self, owner_user_id: &str, req: &PutOwnerAudience) -> Result<OwnerAudience> {
        self.client
            .put_owner_audience(&self.org_id, &self.app_id, owner_user_id, req)
            .await
    }

    /// The reader sign-up (API key).
    pub async fn create_subscriber(
        &self,
        owner_user_id: &str,
        req: &CreateOwnerSubscriber,
    ) -> Result<Accepted> {
        self.client
            .create_owner_subscriber(&self.org_id, &self.app_id, owner_user_id, req)
            .await
    }

    /// Create a draft issue (API key).
    pub async fn create_issue(
        &self,
        owner_user_id: &str,
        req: &CreateOwnerIssue,
    ) -> Result<OwnerIssue> {
        self.client
            .create_owner_issue(&self.org_id, &self.app_id, owner_user_id, req)
            .await
    }

    /// Test-send an issue to the owner's verified email (API key).
    pub async fn test_issue(
        &self,
        owner_user_id: &str,
        issue_id: &str,
        to: &str,
    ) -> Result<Accepted> {
        self.client
            .test_owner_issue(&self.org_id, &self.app_id, owner_user_id, issue_id, to)
            .await
    }

    /// Send an issue to the owner's subscribers (API key).
    pub async fn send_issue(&self, owner_user_id: &str, issue_id: &str) -> Result<Accepted> {
        self.client
            .send_owner_issue(&self.org_id, &self.app_id, owner_user_id, issue_id)
            .await
    }

    /// The signed-in owner's own audience (bearer).
    pub async fn me(&self, collection: &str) -> Result<OwnerAudience> {
        self.client
            .me_audience(&self.org_id, &self.app_id, collection)
            .await
    }

    /// The signed-in owner's subscribers (bearer).
    pub async fn me_subscribers(
        &self,
        collection: &str,
        q: &SubscriberQuery,
    ) -> Result<Page<OwnerSubscriber>> {
        self.client
            .me_audience_subscribers(&self.org_id, &self.app_id, collection, q)
            .await
    }

    /// The filtered subscribers as CSV (bearer).
    pub async fn me_subscribers_csv(
        &self,
        collection: &str,
        q: &SubscriberQuery,
    ) -> Result<String> {
        self.client
            .me_audience_subscribers_csv(&self.org_id, &self.app_id, collection, q)
            .await
    }

    /// Erase one subscriber (bearer).
    pub async fn me_delete_subscriber(&self, collection: &str, id: &str) -> Result<()> {
        self.client
            .me_delete_audience_subscriber(&self.org_id, &self.app_id, collection, id)
            .await
    }

    /// The signed-in owner's issues (bearer).
    pub async fn me_issues(
        &self,
        collection: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Page<OwnerIssue>> {
        self.client
            .me_audience_issues(&self.org_id, &self.app_id, collection, limit, offset)
            .await
    }
}
