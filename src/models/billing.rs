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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBillingCatalog {
    pub plans: Vec<AppBillingPlan>,
    pub addons: Vec<AppBillingAddon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBillingPlan {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub price_monthly: i64,
    #[serde(default)]
    pub price_yearly: Option<i64>,
    pub currency: String,
    #[serde(default)]
    pub features: serde_json::Value,
    #[serde(default)]
    pub included_addons: Vec<String>,
    pub active: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBillingAddon {
    pub feature_key: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub price_cents: i64,
    pub currency: String,
    #[serde(default)]
    pub included_in_plans: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingIntentCreate {
    pub email: String,
    pub plan_slug: String,
    #[serde(default)]
    pub addon_keys: Vec<String>,
    #[serde(default)]
    pub success_url: Option<String>,
    #[serde(default)]
    pub cancel_url: Option<String>,
    #[serde(default)]
    pub collection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingIntentCheckoutRequest {
    #[serde(default)]
    pub redirect_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingIntentResponse {
    pub id: String,
    pub org_id: String,
    pub app_id: String,
    pub email: String,
    pub plan_slug: String,
    pub addon_keys: Vec<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    #[serde(default)]
    pub checkout_url: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingIntentCheckoutResponse {
    pub intent_id: String,
    #[serde(default)]
    pub payment_id: Option<String>,
    #[serde(default)]
    pub checkout_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWithBillingIntentRequest {
    pub billing_intent_id: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageStatus {
    pub period_start: String,
    pub period_end: String,
    pub currency: String,
    pub budget_cents: i64,
    pub used_cents: i64,
    pub remaining_cents: i64,
    pub remaining_ratio: f64,
    pub allowed_models: Vec<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub suggested_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageCheckRequest {
    pub model: String,
    #[serde(default)]
    pub estimated_input_tokens: Option<i64>,
    #[serde(default)]
    pub estimated_output_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageCheckResponse {
    pub allowed: bool,
    #[serde(default)]
    pub reason: Option<String>,
    pub status: AiUsageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageReportRequest {
    pub model: String,
    pub operation: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    #[serde(default)]
    pub estimated: bool,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageReportResponse {
    pub event_id: String,
    pub cost_micro_eur: i64,
    pub status: AiUsageStatus,
}
