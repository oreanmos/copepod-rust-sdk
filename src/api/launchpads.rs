use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    Launchpad, LaunchpadLaunchRequest, LaunchpadLaunchResponse, SourceDetectionResult,
};

impl CopepodClient {
    /// List launchpads for an organization.
    pub async fn list_launchpads(&self, org_id: &str) -> Result<Vec<Launchpad>> {
        self.get(&format!("api/platform/orgs/{}/launchpads", org_id))
            .await
    }

    /// Get a launchpad by ID.
    pub async fn get_launchpad(&self, org_id: &str, launchpad_id: &str) -> Result<Launchpad> {
        self.get(&format!(
            "api/platform/orgs/{}/launchpads/{}",
            org_id, launchpad_id
        ))
        .await
    }

    /// Create a launchpad.
    pub async fn create_launchpad(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Launchpad> {
        self.post(&format!("api/platform/orgs/{}/launchpads", org_id), body)
            .await
    }

    /// Update an existing launchpad.
    pub async fn update_launchpad(
        &self,
        org_id: &str,
        launchpad_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Launchpad> {
        self.patch(
            &format!("api/platform/orgs/{}/launchpads/{}", org_id, launchpad_id),
            body,
        )
        .await
    }

    /// Delete a launchpad.
    pub async fn delete_launchpad(&self, org_id: &str, launchpad_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/launchpads/{}",
            org_id, launchpad_id
        ))
        .await
    }

    /// Publish the current draft definition of a launchpad.
    pub async fn publish_launchpad(&self, org_id: &str, launchpad_id: &str) -> Result<Launchpad> {
        self.post(
            &format!(
                "api/platform/orgs/{}/launchpads/{}/publish",
                org_id, launchpad_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Detect source settings from launchpad values before launch.
    pub async fn detect_launchpad_source(
        &self,
        org_id: &str,
        launchpad_id: &str,
        body: &LaunchpadLaunchRequest,
    ) -> Result<SourceDetectionResult> {
        self.post(
            &format!(
                "api/platform/orgs/{}/launchpads/{}/detect-source",
                org_id, launchpad_id
            ),
            body,
        )
        .await
    }

    /// Launch a published launchpad.
    pub async fn launch_launchpad(
        &self,
        org_id: &str,
        launchpad_id: &str,
        body: &LaunchpadLaunchRequest,
    ) -> Result<LaunchpadLaunchResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{}/launchpads/{}/launch",
                org_id, launchpad_id
            ),
            body,
        )
        .await
    }
}
