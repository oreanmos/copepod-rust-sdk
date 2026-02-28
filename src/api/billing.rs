use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{CheckoutSession, Plan, Subscription};

impl CopepodClient {
    /// Get billing/subscription status for an organization.
    pub async fn get_billing_status(&self, org_id: &str) -> Result<Subscription> {
        self.get(&format!("api/orgs/{}/billing", org_id)).await
    }

    /// Create a checkout session for an organization.
    pub async fn create_checkout(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<CheckoutSession> {
        self.post(&format!("api/orgs/{}/billing/checkout", org_id), body)
            .await
    }

    /// List available plans for an organization.
    pub async fn list_plans(&self, org_id: &str) -> Result<Vec<Plan>> {
        self.get(&format!("api/orgs/{}/billing/plans", org_id))
            .await
    }

    /// Update the plan for a subscription.
    pub async fn update_plan(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(&format!("api/orgs/{}/billing/plan", org_id), body)
            .await
    }

    /// Cancel the subscription for an organization.
    pub async fn cancel_subscription(&self, org_id: &str) -> Result<()> {
        self.post_empty(
            &format!("api/orgs/{}/billing/cancel", org_id),
            &serde_json::json!({}),
        )
        .await
    }
}
