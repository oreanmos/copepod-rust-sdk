//! SDK client methods for the plan-addon framework.

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AddonCatalogEntry, FeatureDefinition, FeatureDefinitionPatch, ItemsResponse, PlanAddon,
    PlanAddonInput, SubscriptionAddon, SubscriptionAddonInput,
};
use serde_json::json;

impl CopepodClient {
    // ---- catalog ----

    /// List billable addons for an app, optionally annotated with whether
    /// each is bundled in the supplied plan id.
    pub async fn list_app_addons(
        &self,
        org_id: &str,
        app_id: &str,
        plan_id: Option<&str>,
    ) -> Result<ItemsResponse<AddonCatalogEntry>> {
        let mut path = format!("api/platform/orgs/{org_id}/apps/{app_id}/addons");
        if let Some(pid) = plan_id {
            path.push_str(&format!("?plan_id={pid}"));
        }
        self.get(&path).await
    }

    /// Patch a feature definition (used to flip `billable`, set price, etc.).
    pub async fn patch_feature_definition(
        &self,
        org_id: &str,
        app_id: &str,
        key: &str,
        patch: &FeatureDefinitionPatch,
    ) -> Result<FeatureDefinition> {
        self.patch(
            &format!("api/platform/orgs/{org_id}/apps/{app_id}/features/{key}"),
            patch,
        )
        .await
    }

    // ---- plan inclusion matrix ----

    pub async fn list_plan_addons(&self, plan_id: &str) -> Result<ItemsResponse<PlanAddon>> {
        self.get(&format!("api/platform/plans/{plan_id}/addons"))
            .await
    }

    pub async fn set_plan_addons(
        &self,
        plan_id: &str,
        items: &[PlanAddonInput],
    ) -> Result<ItemsResponse<PlanAddon>> {
        self.put(
            &format!("api/platform/plans/{plan_id}/addons"),
            &json!({ "items": items }),
        )
        .await
    }

    // ---- per-org subscription cart ----

    pub async fn list_subscription_addons(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ItemsResponse<SubscriptionAddon>> {
        self.get(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/subscription/addons"
        ))
        .await
    }

    pub async fn add_subscription_addon(
        &self,
        org_id: &str,
        app_id: &str,
        feature_key: &str,
    ) -> Result<SubscriptionAddon> {
        self.post(
            &format!("api/platform/orgs/{org_id}/apps/{app_id}/subscription/addons"),
            &SubscriptionAddonInput {
                feature_key: feature_key.to_string(),
            },
        )
        .await
    }

    pub async fn cancel_subscription_addon(
        &self,
        org_id: &str,
        app_id: &str,
        feature_key: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/subscription/addons/{feature_key}"
        ))
        .await
    }
}
