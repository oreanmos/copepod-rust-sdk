use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCreate {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub price_monthly: Option<i64>,
    #[serde(default)]
    pub price_yearly: Option<i64>,
    #[serde(default)]
    pub features: Option<serde_json::Value>,
    #[serde(default)]
    pub limits: Option<serde_json::Value>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub mollie_plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub price_monthly: Option<i64>,
    #[serde(default)]
    pub price_yearly: Option<i64>,
    #[serde(default)]
    pub features: Option<serde_json::Value>,
    #[serde(default)]
    pub limits: Option<serde_json::Value>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub mollie_plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub id: String,
    pub org_id: String,
    pub feature_key: String,
    pub limit_value: String,
    pub source: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementInput {
    pub feature_key: String,
    pub limit_value: String,
}
