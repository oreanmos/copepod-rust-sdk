use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effective feature map: feature_key -> resolved value.
pub type FeatureMap = HashMap<String, String>;

/// A feature registered by an app in the feature catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub id: String,
    pub app_id: String,
    pub key: String,
    pub value_type: String,
    pub scope: String,
    pub default_value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
    #[serde(default)]
    pub billable: bool,
    #[serde(default)]
    pub price_cents: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub mollie_addon_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    pub created: String,
}

/// Input for creating a feature definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinitionCreate {
    pub key: String,
    pub value_type: String,
    pub scope: String,
    pub default_value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
    #[serde(default)]
    pub billable: bool,
    #[serde(default)]
    pub price_cents: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub mollie_addon_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Patch payload for an existing feature definition. Each field is
/// `Some(...)` when the caller wants to set it, `None` to leave unchanged.
/// Inner `Option<T>` allows setting a column to NULL.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureDefinitionPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<Option<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mollie_addon_id: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
}

/// A mapping from a plan to a feature value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFeatureMapping {
    pub id: String,
    pub plan_id: String,
    pub feature_key: String,
    pub value: String,
    pub created: String,
}

/// Input for setting a plan feature mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFeatureMappingInput {
    pub feature_key: String,
    pub value: String,
}

/// A grant of a feature value to a subject (org or user).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureGrant {
    pub id: String,
    #[serde(default)]
    pub feature_def_id: Option<String>,
    pub subject_type: String,
    pub subject_id: String,
    pub app_id: String,
    pub feature_key: String,
    pub value: String,
    pub source: String,
    #[serde(default)]
    pub source_id: Option<String>,
    pub created: String,
    pub updated: String,
}

/// Input for creating/updating a manual feature grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureGrantInput {
    pub subject_type: String,
    pub subject_id: String,
    pub feature_key: String,
    pub value: String,
}

/// Response from feature resolution endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFeatures {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub subject_type: Option<String>,
    #[serde(default)]
    pub subject_id: Option<String>,
    pub app_id: String,
    pub features: FeatureMap,
}
