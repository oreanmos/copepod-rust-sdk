use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A deployed application (container workload managed by the PaaS layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub slug: String,
    pub image: String,
    pub tag: String,
    pub replicas: u32,
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
    pub port: u16,
    pub health_check_path: String,
    pub status: DeploymentStatus,
    pub last_deployed_at: Option<DateTime<Utc>>,
    pub current_image: Option<String>,
    pub namespace: String,
    pub webhook_token: Option<String>,
    pub webhook_enabled: Option<bool>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Status of a deployed application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Running,
    Stopped,
    Failed,
}

/// Request body to create a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCreate {
    pub name: String,
    pub slug: String,
    pub image: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub replicas: Option<u32>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub cpu_request: Option<String>,
    #[serde(default)]
    pub cpu_limit: Option<String>,
    #[serde(default)]
    pub memory_request: Option<String>,
    #[serde(default)]
    pub memory_limit: Option<String>,
    #[serde(default)]
    pub health_check_path: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
}

/// Request body to update a deployment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub replicas: Option<u32>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub cpu_request: Option<String>,
    #[serde(default)]
    pub cpu_limit: Option<String>,
    #[serde(default)]
    pub memory_request: Option<String>,
    #[serde(default)]
    pub memory_limit: Option<String>,
    #[serde(default)]
    pub health_check_path: Option<String>,
}

/// An environment variable attached to a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEnvVar {
    pub id: String,
    pub app_id: String,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub created: DateTime<Utc>,
}

/// Input for setting an environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarInput {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_secret: bool,
}

/// Bulk env var set request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarBulkSet {
    pub vars: Vec<EnvVarInput>,
}

/// A custom domain attached to a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentDomain {
    pub id: String,
    pub app_id: String,
    pub host: String,
    pub path: String,
    pub port: u16,
    pub https_enabled: bool,
    pub cert_auto: bool,
    pub created: DateTime<Utc>,
}

/// Input for adding a custom domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInput {
    pub host: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub https_enabled: Option<bool>,
    #[serde(default)]
    pub cert_auto: Option<bool>,
}

/// A persistent volume attached to a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentVolume {
    pub id: String,
    pub app_id: String,
    pub mount_path: String,
    pub size: String,
    pub storage_class: String,
    pub created: DateTime<Utc>,
}

/// Input for adding a persistent volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInput {
    pub mount_path: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub storage_class: Option<String>,
}

/// A deployment history log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentHistoryEntry {
    pub id: String,
    pub app_id: String,
    pub action: String,
    pub status: String,
    pub message: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub triggered_by: Option<String>,
    pub created: DateTime<Utc>,
}

/// Container log output from a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentLogs {
    pub lines: Vec<String>,
}

/// Resource metrics for a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub cpu_usage: String,
    pub memory_usage: String,
    pub ready_replicas: u32,
    pub total_replicas: u32,
}

/// Runtime status for a deployment, sourced from the active deploy backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRuntimeStatus {
    pub running: bool,
    pub ready_replicas: u32,
    pub desired_replicas: u32,
    pub message: String,
    pub db_status: DeploymentStatus,
}

/// Webhook configuration for auto-redeploy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentWebhook {
    pub webhook_token: Option<String>,
    pub webhook_enabled: bool,
    pub webhook_url: Option<String>,
}

/// Queue acknowledgement for async deployment actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentQueueAck {
    pub queued: bool,
    pub app_id: String,
    pub log_id: String,
    pub action: String,
    pub status: String,
}

/// Supported git providers for source deployments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentGitProvider {
    Github,
    Gitlab,
    Bitbucket,
    Gitea,
}

/// Supported git auth methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentGitAuthMethod {
    None,
    DeployKey,
    GithubApp,
    Token,
}

/// Build method for source-based deployments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentBuildMethod {
    Nixpacks,
    Dockerfile,
    DockerCompose,
}

/// Git source configuration linked to a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGitSource {
    pub id: String,
    pub deployed_app_id: String,
    pub provider: DeploymentGitProvider,
    pub repo_url: String,
    pub branch: String,
    pub auth_method: DeploymentGitAuthMethod,
    pub credentials_encrypted: Option<String>,
    pub webhook_secret: Option<String>,
    pub github_installation_id: Option<i64>,
    pub dockerfile_path: String,
    pub build_context: String,
    pub build_method: Option<DeploymentBuildMethod>,
    pub auto_deploy: bool,
    pub created: String,
    pub updated: String,
}

/// Input for creating a git source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGitSourceCreate {
    pub provider: DeploymentGitProvider,
    pub repo_url: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub auth_method: Option<DeploymentGitAuthMethod>,
    #[serde(default)]
    pub credentials: Option<String>,
    #[serde(default)]
    pub github_installation_id: Option<i64>,
    #[serde(default)]
    pub dockerfile_path: Option<String>,
    #[serde(default)]
    pub build_context: Option<String>,
    #[serde(default)]
    pub build_method: Option<DeploymentBuildMethod>,
    #[serde(default)]
    pub auto_deploy: Option<bool>,
}

/// Input for updating a git source.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentGitSourceUpdate {
    #[serde(default)]
    pub repo_url: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub auth_method: Option<DeploymentGitAuthMethod>,
    #[serde(default)]
    pub credentials: Option<String>,
    #[serde(default)]
    pub github_installation_id: Option<i64>,
    #[serde(default)]
    pub dockerfile_path: Option<String>,
    #[serde(default)]
    pub build_context: Option<String>,
    #[serde(default)]
    pub build_method: Option<DeploymentBuildMethod>,
    #[serde(default)]
    pub auto_deploy: Option<bool>,
}

/// Response from creating a git source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGitSourceCreateResponse {
    pub git_source: DeploymentGitSource,
    pub webhook_url: String,
    pub webhook_secret: Option<String>,
}

/// Build status for source builds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentBuildStatus {
    Pending,
    Cloning,
    Building,
    Pushing,
    Success,
    Failed,
    Cancelled,
}

/// Build job metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentBuildJob {
    pub id: String,
    pub deployed_app_id: String,
    pub git_source_id: String,
    pub commit_sha: String,
    pub commit_message: String,
    pub branch: String,
    pub status: DeploymentBuildStatus,
    pub build_method: String,
    pub image_tag: String,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created: String,
    pub updated: String,
}

/// Single build log line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentBuildLogLine {
    pub line: i64,
    pub content: String,
    pub stream: String,
}

/// Build details including logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentBuildDetails {
    pub build: DeploymentBuildJob,
    pub logs: Vec<DeploymentBuildLogLine>,
}

/// List wrapper for build jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentBuildList {
    pub items: Vec<DeploymentBuildJob>,
}

/// Trigger-build response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentBuildTriggerResponse {
    pub build_id: String,
    pub queue: DeploymentQueueAck,
}
