use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{FeatureDefinition, FeatureDefinitionCreate, ItemsResponse, ResolvedFeatures};

impl CopepodClient {
    /// Register a feature definition for an app.
    pub async fn create_feature_definition(
        &self,
        org_id: &str,
        app_id: &str,
        body: &FeatureDefinitionCreate,
    ) -> Result<FeatureDefinition> {
        self.post(
            &format!(
                "api/platform/orgs/{}/apps/{}/features",
                org_id, app_id
            ),
            body,
        )
        .await
    }

    /// List feature definitions for an app.
    pub async fn list_feature_definitions(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ItemsResponse<FeatureDefinition>> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/features",
            org_id, app_id
        ))
        .await
    }

    /// Delete a feature definition.
    pub async fn delete_feature_definition(
        &self,
        org_id: &str,
        app_id: &str,
        key: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/features/{}",
            org_id, app_id, key
        ))
        .await
    }

    /// Resolve effective features for a specific user.
    pub async fn resolve_user_features(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
    ) -> Result<ResolvedFeatures> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/users/{}/features",
            org_id, app_id, user_id
        ))
        .await
    }

    /// Resolve effective features for any subject (org or user).
    pub async fn resolve_features(
        &self,
        org_id: &str,
        app_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<ResolvedFeatures> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/features/resolve?subject_type={}&subject_id={}",
            org_id, app_id, subject_type, subject_id
        ))
        .await
    }
}
