use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{FeatureGrant, FeatureGrantInput, ItemsResponse};

impl CopepodClient {
    /// Set a manual feature grant.
    pub async fn set_feature_grant(
        &self,
        org_id: &str,
        app_id: &str,
        body: &FeatureGrantInput,
    ) -> Result<FeatureGrant> {
        self.put(
            &format!(
                "api/platform/orgs/{}/apps/{}/grants",
                org_id, app_id
            ),
            body,
        )
        .await
    }

    /// List feature grants for a subject.
    pub async fn list_feature_grants(
        &self,
        org_id: &str,
        app_id: &str,
        subject_type: &str,
        subject_id: &str,
    ) -> Result<ItemsResponse<FeatureGrant>> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/grants?subject_type={}&subject_id={}",
            org_id, app_id, subject_type, subject_id
        ))
        .await
    }

    /// Delete a feature grant.
    pub async fn delete_feature_grant(
        &self,
        org_id: &str,
        app_id: &str,
        grant_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/grants/{}",
            org_id, app_id, grant_id
        ))
        .await
    }
}
