use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AbuseSignalReceipt, AbuseSignalRequest, AppUserTrust, ContentReportPage, ContentReportQuery,
    ContentReportReceipt, ContentReportRequest, FlaggedAppUsers, ModerateRequest, ModerateResponse,
    ModerationOutcomeRequest, ModerationOutcomeResponse, MyRestrictions, ResolveReviewRequest,
    ResolveReviewResponse, RestrictionScope, ReviewQueuePage, ReviewQueueQuery, SuspensionState,
    TrustActionPage, TrustDecisionPage, TrustDecisionQuery, WrongDecisionRequest,
    WrongDecisionResponse,
};

/// Trust and safety helpers bound to an application. Each method calls the
/// [`CopepodClient`] method of the same name; see there for auth and errors.
#[derive(Debug, Clone)]
pub struct ScopedTrustClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedTrustClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: &str, app_id: &str) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// See [`CopepodClient::moderate`] (API key).
    pub async fn moderate(&self, request: &ModerateRequest) -> Result<ModerateResponse> {
        self.client
            .moderate(&self.org_id, &self.app_id, request)
            .await
    }

    /// See [`CopepodClient::moderate_as_user`] (app-user bearer).
    pub async fn moderate_as_user(
        &self,
        collection: &str,
        request: &ModerateRequest,
    ) -> Result<ModerateResponse> {
        self.client
            .moderate_as_user(&self.org_id, &self.app_id, collection, request)
            .await
    }

    /// See [`CopepodClient::record_moderation_outcome`] (API key).
    pub async fn record_moderation_outcome(
        &self,
        decision_id: &str,
        request: &ModerationOutcomeRequest,
    ) -> Result<ModerationOutcomeResponse> {
        self.client
            .record_moderation_outcome(&self.org_id, &self.app_id, decision_id, request)
            .await
    }

    /// See [`CopepodClient::submit_content_report`] (API key).
    pub async fn submit_content_report(
        &self,
        request: &ContentReportRequest,
    ) -> Result<ContentReportReceipt> {
        self.client
            .submit_content_report(&self.org_id, &self.app_id, request)
            .await
    }

    /// See [`CopepodClient::list_trust_actions`] (API key).
    pub async fn list_trust_actions(
        &self,
        after: Option<i64>,
        limit: u32,
    ) -> Result<TrustActionPage> {
        self.client
            .list_trust_actions(&self.org_id, &self.app_id, after, limit)
            .await
    }

    /// See [`CopepodClient::record_abuse_signal`] (API key).
    pub async fn record_abuse_signal(
        &self,
        request: &AbuseSignalRequest,
    ) -> Result<AbuseSignalReceipt> {
        self.client
            .record_abuse_signal(&self.org_id, &self.app_id, request)
            .await
    }

    /// See [`CopepodClient::my_restrictions`] (app-user bearer).
    pub async fn my_restrictions(&self, collection: &str) -> Result<MyRestrictions> {
        self.client
            .my_restrictions(&self.org_id, &self.app_id, collection)
            .await
    }

    /// See [`CopepodClient::list_review_queue`] (org admin).
    pub async fn list_review_queue(&self, q: &ReviewQueueQuery) -> Result<ReviewQueuePage> {
        self.client
            .list_review_queue(&self.org_id, &self.app_id, q)
            .await
    }

    /// See [`CopepodClient::resolve_review_item`] (org admin).
    pub async fn resolve_review_item(
        &self,
        item_id: &str,
        request: &ResolveReviewRequest,
    ) -> Result<ResolveReviewResponse> {
        self.client
            .resolve_review_item(&self.org_id, &self.app_id, item_id, request)
            .await
    }

    /// See [`CopepodClient::list_trust_decisions`] (org admin).
    pub async fn list_trust_decisions(&self, q: &TrustDecisionQuery) -> Result<TrustDecisionPage> {
        self.client
            .list_trust_decisions(&self.org_id, &self.app_id, q)
            .await
    }

    /// See [`CopepodClient::mark_decision_wrong`] (org admin).
    pub async fn mark_decision_wrong(
        &self,
        decision_id: &str,
        request: &WrongDecisionRequest,
    ) -> Result<WrongDecisionResponse> {
        self.client
            .mark_decision_wrong(&self.org_id, &self.app_id, decision_id, request)
            .await
    }

    /// See [`CopepodClient::list_content_reports`] (org admin).
    pub async fn list_content_reports(&self, q: &ContentReportQuery) -> Result<ContentReportPage> {
        self.client
            .list_content_reports(&self.org_id, &self.app_id, q)
            .await
    }

    /// See [`CopepodClient::get_app_user_trust`] (org admin).
    pub async fn get_app_user_trust(&self, user_id: &str) -> Result<AppUserTrust> {
        self.client
            .get_app_user_trust(&self.org_id, &self.app_id, user_id)
            .await
    }

    /// See [`CopepodClient::list_flagged_app_users`] (org admin).
    pub async fn list_flagged_app_users(&self) -> Result<FlaggedAppUsers> {
        self.client
            .list_flagged_app_users(&self.org_id, &self.app_id)
            .await
    }

    /// See [`CopepodClient::lift_app_user_restriction`] (org admin).
    pub async fn lift_app_user_restriction(
        &self,
        user_id: &str,
        scope: RestrictionScope,
    ) -> Result<()> {
        self.client
            .lift_app_user_restriction(&self.org_id, &self.app_id, user_id, scope)
            .await
    }

    /// See [`CopepodClient::suspend_app_user`] (org admin).
    pub async fn suspend_app_user(&self, user_id: &str, reason: &str) -> Result<SuspensionState> {
        self.client
            .suspend_app_user(&self.org_id, &self.app_id, user_id, reason)
            .await
    }

    /// See [`CopepodClient::unsuspend_app_user`] (org admin).
    pub async fn unsuspend_app_user(&self, user_id: &str) -> Result<SuspensionState> {
        self.client
            .unsuspend_app_user(&self.org_id, &self.app_id, user_id)
            .await
    }
}
