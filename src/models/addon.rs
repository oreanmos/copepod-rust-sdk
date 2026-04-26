//! Plan-addon framework SDK models.
//!
//! Mirrors copepod's `copepod_models::addon` module for use by:
//! * SDK consumers managing addons (admin tools, cli);
//! * Apps publishing a feature manifest at `/__copepod/feature-manifest`.

use serde::{Deserialize, Serialize};

/// Per-plan declaration of which addons are bundled at no extra cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAddon {
    pub plan_id: String,
    pub feature_key: String,
    pub included_by_default: bool,
    pub created: String,
}

/// Input for setting plan addon inclusions in bulk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAddonInput {
    pub feature_key: String,
    #[serde(default = "default_true")]
    pub included_by_default: bool,
}

fn default_true() -> bool {
    true
}

/// A purchased addon line on an org's subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionAddon {
    pub id: String,
    pub org_id: String,
    pub app_id: String,
    pub feature_key: String,
    pub status: String,
    pub price_cents: i64,
    pub currency: String,
    pub started: String,
    #[serde(default)]
    pub cancelled: Option<String>,
    #[serde(default)]
    pub mollie_line_id: Option<String>,
    pub created: String,
    pub updated: String,
}

/// Body for adding an addon to an org's subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionAddonInput {
    pub feature_key: String,
}

/// Compact catalog row, as returned by `GET /orgs/.../apps/.../addons`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonCatalogEntry {
    pub feature_key: String,
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub price_cents: i64,
    pub currency: String,
    #[serde(default)]
    pub mollie_addon_id: Option<String>,
    #[serde(default)]
    pub included_in_plan: Option<bool>,
}

/// App-published feature manifest. Apps that want to be addon-aware should
/// expose this at `GET /__copepod/feature-manifest` and copepod will sync it
/// after every successful deploy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureManifest {
    pub app_slug: String,
    pub schema_version: u32,
    pub features: Vec<ManifestFeature>,
}

/// One feature entry in a [`FeatureManifest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFeature {
    pub key: String,
    pub value_type: String,
    pub scope: String,
    pub default_value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub billable: bool,
    #[serde(default)]
    pub suggested_price_cents: Option<i64>,
    #[serde(default)]
    pub suggested_currency: Option<String>,
    #[serde(default)]
    pub included_in_plans: Vec<String>,
}
