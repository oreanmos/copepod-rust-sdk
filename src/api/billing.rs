use reqwest::Method;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AppPlanChangePreview, AppPlanChangeRequest, AppPlanChangeResponse, CheckoutSession, Plan,
    Subscription,
};

impl CopepodClient {
    /// Get billing/subscription status for an organization.
    pub async fn get_billing_status(&self, org_id: &str) -> Result<Subscription> {
        self.get(&format!("api/platform/orgs/{}/billing", org_id))
            .await
    }

    /// Create a checkout session for an organization.
    pub async fn create_checkout(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<CheckoutSession> {
        self.post(
            &format!("api/platform/orgs/{}/billing/checkout", org_id),
            body,
        )
        .await
    }

    /// List available plans for an organization.
    pub async fn list_plans(&self, org_id: &str) -> Result<Vec<Plan>> {
        self.get(&format!("api/platform/orgs/{}/billing/plans", org_id))
            .await
    }

    /// Update the plan for a subscription.
    pub async fn update_plan(&self, org_id: &str, body: &impl serde::Serialize) -> Result<Value> {
        self.put(&format!("api/platform/orgs/{}/billing/plan", org_id), body)
            .await
    }

    /// Cancel the subscription for an organization.
    pub async fn cancel_subscription(&self, org_id: &str) -> Result<()> {
        self.post_empty(
            &format!("api/platform/orgs/{}/billing/cancel", org_id),
            &serde_json::json!({}),
        )
        .await
    }

    /// List payment history for an organization.
    pub async fn list_payments(&self, org_id: &str) -> Result<Value> {
        self.get(&format!("api/platform/orgs/{}/billing/payments", org_id))
            .await
    }

    /// Get usage metrics for an organization.
    pub async fn get_usage(&self, org_id: &str) -> Result<Value> {
        self.get(&format!("api/platform/orgs/{}/usage", org_id))
            .await
    }

    /// Update entitlements for an organization (billing context).
    pub async fn update_billing_entitlements(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!("api/platform/orgs/{}/billing/entitlements", org_id),
            body,
        )
        .await
    }

    /// Preview an app-user plan change using the caller's app-user bearer token.
    pub async fn preview_app_user_plan_change(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        target_plan: &str,
    ) -> Result<AppPlanChangePreview> {
        self.get(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/subscription/change-preview?target_plan={target_plan}"
        ))
        .await
    }

    /// Submit an app-user plan change using the caller's app-user bearer token.
    pub async fn change_app_user_plan(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &AppPlanChangeRequest,
    ) -> Result<AppPlanChangeResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/subscription/change"
            ),
            body,
        )
        .await
    }

    /// Cancel the current app-user's scheduled downgrade.
    pub async fn cancel_app_user_plan_change(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<AppPlanChangeResponse> {
        let resp = self
            .auth_request(
                Method::DELETE,
                &format!(
                    "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/subscription/change"
                ),
            )
            .await?
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }
}
