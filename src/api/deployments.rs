use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    Deployment, DeploymentBuildDetails, DeploymentBuildJob, DeploymentBuildList,
    DeploymentBuildTriggerResponse, DeploymentDomain, DeploymentEnvVar, DeploymentGitSource,
    DeploymentGitSourceCreateResponse, DeploymentHistoryEntry, DeploymentLogs, DeploymentMetrics,
    DeploymentQueueAck, DeploymentRuntimeStatus, DeploymentVolume, DeploymentWebhook,
    SourceDetectionResult,
};

impl CopepodClient {
    // -- Deployments CRUD --

    /// List all deployments in an organization.
    pub async fn list_deployments(&self, org_id: &str) -> Result<Vec<Deployment>> {
        self.get(&format!("api/platform/orgs/{}/deployments", org_id))
            .await
    }

    /// Get a deployment by ID.
    pub async fn get_deployment(&self, org_id: &str, deploy_id: &str) -> Result<Deployment> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}",
            org_id, deploy_id
        ))
        .await
    }

    /// Create a new deployment.
    pub async fn create_deployment(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Deployment> {
        self.post(&format!("api/platform/orgs/{}/deployments", org_id), body)
            .await
    }

    /// Update a deployment.
    pub async fn update_deployment(
        &self,
        org_id: &str,
        deploy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Deployment> {
        self.patch(
            &format!("api/platform/orgs/{}/deployments/{}", org_id, deploy_id),
            body,
        )
        .await
    }

    /// Delete a deployment.
    pub async fn delete_deployment(&self, org_id: &str, deploy_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/deployments/{}",
            org_id, deploy_id
        ))
        .await
    }

    // -- Deploy actions --

    /// Trigger a deploy (build + rollout).
    pub async fn deploy(&self, org_id: &str, deploy_id: &str) -> Result<()> {
        self.deploy_queued(org_id, deploy_id).await.map(|_| ())
    }

    /// Trigger a deploy and return queue metadata.
    pub async fn deploy_queued(&self, org_id: &str, deploy_id: &str) -> Result<DeploymentQueueAck> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/deploy",
                org_id, deploy_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Stop a running deployment.
    pub async fn stop_deployment(&self, org_id: &str, deploy_id: &str) -> Result<()> {
        self.stop_deployment_queued(org_id, deploy_id)
            .await
            .map(|_| ())
    }

    /// Stop a deployment and return queue metadata.
    pub async fn stop_deployment_queued(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentQueueAck> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/stop",
                org_id, deploy_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Start a stopped deployment.
    pub async fn start_deployment(&self, org_id: &str, deploy_id: &str) -> Result<()> {
        self.start_deployment_queued(org_id, deploy_id)
            .await
            .map(|_| ())
    }

    /// Start a deployment and return queue metadata.
    pub async fn start_deployment_queued(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentQueueAck> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/start",
                org_id, deploy_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    // -- Env vars --

    /// List environment variables for a deployment.
    pub async fn list_deployment_env(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<Vec<DeploymentEnvVar>> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/env",
            org_id, deploy_id
        ))
        .await
    }

    /// Set environment variables in bulk.
    pub async fn set_deployment_env(
        &self,
        org_id: &str,
        deploy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Vec<DeploymentEnvVar>> {
        self.put(
            &format!("api/platform/orgs/{}/deployments/{}/env", org_id, deploy_id),
            body,
        )
        .await
    }

    /// Delete a single environment variable by key.
    pub async fn delete_deployment_env_var(
        &self,
        org_id: &str,
        deploy_id: &str,
        key: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/deployments/{}/env/{}",
            org_id, deploy_id, key
        ))
        .await
    }

    // -- Domains --

    /// List custom domains for a deployment.
    pub async fn list_deployment_domains(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<Vec<DeploymentDomain>> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/domains",
            org_id, deploy_id
        ))
        .await
    }

    /// Add a custom domain to a deployment.
    pub async fn add_deployment_domain(
        &self,
        org_id: &str,
        deploy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<DeploymentDomain> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/domains",
                org_id, deploy_id
            ),
            body,
        )
        .await
    }

    /// Remove a custom domain from a deployment.
    pub async fn delete_deployment_domain(
        &self,
        org_id: &str,
        deploy_id: &str,
        domain_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/deployments/{}/domains/{}",
            org_id, deploy_id, domain_id
        ))
        .await
    }

    // -- Volumes --

    /// List persistent volumes for a deployment.
    pub async fn list_deployment_volumes(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<Vec<DeploymentVolume>> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/volumes",
            org_id, deploy_id
        ))
        .await
    }

    /// Add a persistent volume to a deployment.
    pub async fn add_deployment_volume(
        &self,
        org_id: &str,
        deploy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<DeploymentVolume> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/volumes",
                org_id, deploy_id
            ),
            body,
        )
        .await
    }

    /// Remove a persistent volume from a deployment.
    pub async fn delete_deployment_volume(
        &self,
        org_id: &str,
        deploy_id: &str,
        volume_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/deployments/{}/volumes/{}",
            org_id, deploy_id, volume_id
        ))
        .await
    }

    // -- History --

    /// List deployment history entries.
    pub async fn list_deployment_history(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<Vec<DeploymentHistoryEntry>> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/history",
            org_id, deploy_id
        ))
        .await
    }

    // -- Logs & Metrics --

    /// Fetch recent container logs for a deployment.
    pub async fn get_deployment_logs(
        &self,
        org_id: &str,
        deploy_id: &str,
        tail: Option<u32>,
    ) -> Result<DeploymentLogs> {
        let tail = tail.unwrap_or(200);
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/logs?tail={}",
            org_id, deploy_id, tail
        ))
        .await
    }

    /// Fetch current resource metrics for a deployment.
    pub async fn get_deployment_metrics(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentMetrics> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/metrics",
            org_id, deploy_id
        ))
        .await
    }

    /// Fetch runtime status from the deployment backend/operator view.
    pub async fn get_deployment_status(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentRuntimeStatus> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/status",
            org_id, deploy_id
        ))
        .await
    }

    // -- Webhooks --

    /// Generate or regenerate a webhook token for auto-redeploy.
    pub async fn set_deployment_webhook(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentWebhook> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/webhook",
                org_id, deploy_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Disable the webhook for a deployment.
    pub async fn delete_deployment_webhook(&self, org_id: &str, deploy_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/deployments/{}/webhook",
            org_id, deploy_id
        ))
        .await
    }

    // -- Git source --

    /// Get git source configuration for a deployment.
    pub async fn get_deployment_git_source(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentGitSource> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/git",
            org_id, deploy_id
        ))
        .await
    }

    /// Create git source configuration for a deployment.
    pub async fn create_deployment_git_source(
        &self,
        org_id: &str,
        deploy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<DeploymentGitSourceCreateResponse> {
        self.post(
            &format!("api/platform/orgs/{}/deployments/{}/git", org_id, deploy_id),
            body,
        )
        .await
    }

    /// Update git source configuration for a deployment.
    pub async fn update_deployment_git_source(
        &self,
        org_id: &str,
        deploy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<DeploymentGitSource> {
        self.patch(
            &format!("api/platform/orgs/{}/deployments/{}/git", org_id, deploy_id),
            body,
        )
        .await
    }

    /// Delete git source configuration for a deployment.
    pub async fn delete_deployment_git_source(&self, org_id: &str, deploy_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/deployments/{}/git",
            org_id, deploy_id
        ))
        .await
    }

    // -- Build jobs --

    /// List build jobs for a deployment.
    pub async fn list_deployment_builds(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<Vec<DeploymentBuildJob>> {
        let wrapper: DeploymentBuildList = self
            .get(&format!(
                "api/platform/orgs/{}/deployments/{}/builds",
                org_id, deploy_id
            ))
            .await?;
        Ok(wrapper.items)
    }

    /// Get a build job and its log lines.
    pub async fn get_deployment_build(
        &self,
        org_id: &str,
        deploy_id: &str,
        build_id: &str,
    ) -> Result<DeploymentBuildDetails> {
        self.get(&format!(
            "api/platform/orgs/{}/deployments/{}/builds/{}",
            org_id, deploy_id, build_id
        ))
        .await
    }

    /// Trigger a source build and queue deployment.
    pub async fn trigger_deployment_build(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<DeploymentBuildTriggerResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/builds/trigger",
                org_id, deploy_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// Detect runtime hints from a deployment's configured git source.
    pub async fn detect_deployment_source(
        &self,
        org_id: &str,
        deploy_id: &str,
    ) -> Result<SourceDetectionResult> {
        self.post(
            &format!(
                "api/platform/orgs/{}/deployments/{}/detect",
                org_id, deploy_id
            ),
            &serde_json::json!({}),
        )
        .await
    }
}
