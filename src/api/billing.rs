use reqwest::Method;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AiUsageCheckRequest, AiUsageCheckResponse, AiUsageReportRequest, AiUsageReportResponse,
    AiUsageStatus, AppBillingCatalog, AppPlanChangePreview, AppPlanChangeRequest,
    AppPlanChangeResponse, BillingIntentCheckoutRequest, BillingIntentCheckoutResponse,
    BillingIntentCreate, BillingIntentResponse, CheckoutSession, Plan,
    RegisterWithBillingIntentRequest, Subscription,
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

    /// Fetch the public app billing catalog.
    pub async fn get_app_billing_catalog(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<AppBillingCatalog> {
        self.get_public(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/billing/catalog"
        ))
        .await
    }

    /// Create a public pre-registration billing intent.
    pub async fn create_app_billing_intent(
        &self,
        org_id: &str,
        app_id: &str,
        body: &BillingIntentCreate,
    ) -> Result<BillingIntentResponse> {
        self.post_public(
            &format!("api/platform/orgs/{org_id}/apps/{app_id}/billing/intents"),
            body,
        )
        .await
    }

    /// Fetch a public pre-registration billing intent.
    pub async fn get_app_billing_intent(
        &self,
        org_id: &str,
        app_id: &str,
        intent_id: &str,
    ) -> Result<BillingIntentResponse> {
        self.get_public(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/billing/intents/{intent_id}"
        ))
        .await
    }

    /// Start Mollie checkout for a pre-registration billing intent.
    pub async fn checkout_app_billing_intent(
        &self,
        org_id: &str,
        app_id: &str,
        intent_id: &str,
        body: &BillingIntentCheckoutRequest,
    ) -> Result<BillingIntentCheckoutResponse> {
        self.post_public(
            &format!(
                "api/platform/orgs/{org_id}/apps/{app_id}/billing/intents/{intent_id}/checkout"
            ),
            body,
        )
        .await
    }

    /// Register an app user by consuming a paid billing intent.
    pub async fn register_with_billing_intent(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &RegisterWithBillingIntentRequest,
    ) -> Result<crate::models::AuthResponse> {
        self.post_public(
            &format!(
                "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/register-with-billing-intent"
            ),
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

    /// Fetch the current app user's hosted AI usage status.
    pub async fn get_current_ai_usage(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<AiUsageStatus> {
        self.get(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/ai-usage"
        ))
        .await
    }

    /// Check whether a hosted AI request is allowed for the current app user.
    pub async fn check_current_ai_usage(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &AiUsageCheckRequest,
    ) -> Result<AiUsageCheckResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/ai-usage/check"
            ),
            body,
        )
        .await
    }

    /// Report hosted AI token usage for the current app user.
    pub async fn report_current_ai_usage(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &AiUsageReportRequest,
    ) -> Result<AiUsageReportResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/ai-usage/report"
            ),
            body,
        )
        .await
    }
}
