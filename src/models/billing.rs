use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A billing subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub org_id: String,
    pub plan_id: String,
    pub status: String,
    #[serde(default)]
    pub current_period_end: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Request body to create a checkout session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub plan_id: String,
    #[serde(default)]
    pub success_url: Option<String>,
    #[serde(default)]
    pub cancel_url: Option<String>,
}

/// A checkout session returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: String,
}

/// A billing plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub interval: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

/// Query for previewing an app-user subscription plan change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPlanChangePreviewQuery {
    pub target_plan: String,
}

/// Request body for applying an app-user subscription plan change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPlanChangeRequest {
    pub target_plan: String,
    #[serde(default)]
    pub accepted_terms_version: Option<String>,
    #[serde(default)]
    pub accepted_immediate_service: bool,
    #[serde(default)]
    pub accepted_price: bool,
    #[serde(default)]
    pub redirect_url: Option<String>,
}

/// Pending app-user plan change returned with plan-change previews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAppPlanChange {
    pub target_plan: String,
    pub direction: String,
    pub status: String,
    pub effective_at: String,
}

/// Server-calculated preview for app-user subscription plan changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPlanChangePreview {
    pub current_plan: String,
    pub target_plan: String,
    pub direction: String,
    pub current_period_start: String,
    pub current_period_end: String,
    pub old_recurring_cents: i64,
    pub new_recurring_cents: i64,
    pub prorated_charge_cents: i64,
    pub currency: String,
    pub effective_at: String,
    pub checkout_required: bool,
    #[serde(default)]
    pub pending_change: Option<PendingAppPlanChange>,
}

/// Result of submitting or cancelling an app-user plan change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPlanChangeResponse {
    pub status: String,
    pub preview: AppPlanChangePreview,
    #[serde(default)]
    pub checkout_url: Option<String>,
    #[serde(default)]
    pub payment_id: Option<String>,
}
