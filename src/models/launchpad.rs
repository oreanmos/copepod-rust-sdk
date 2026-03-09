use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Launchpad publication state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchpadStatus {
    Draft,
    Published,
}

/// A reusable launch workflow definition scoped to an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launchpad {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub status: LaunchpadStatus,
    pub version: u32,
    pub draft_definition: LaunchpadDefinition,
    pub published_definition: Option<LaunchpadDefinition>,
    pub published_at: Option<String>,
    pub created: String,
    pub updated: String,
}

/// Request body for creating a launchpad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadCreate {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: String,
    pub definition: LaunchpadDefinition,
}

/// Request body for updating a launchpad.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchpadUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub definition: Option<LaunchpadDefinition>,
}

/// Full launchpad definition that drives the generated launch form.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchpadDefinition {
    #[serde(default)]
    pub headline: String,
    #[serde(default)]
    pub launch_button_label: String,
    #[serde(default)]
    pub create_app: bool,
    #[serde(default)]
    pub app_defaults: LaunchpadAppDefaults,
    #[serde(default)]
    pub deployment_defaults: LaunchpadDeploymentDefaults,
    #[serde(default)]
    pub source_defaults: LaunchpadSourceDefaults,
    #[serde(default)]
    pub domain_defaults: Option<LaunchpadDomainDefaults>,
    #[serde(default)]
    pub static_env: Vec<LaunchpadStaticEnvVar>,
    #[serde(default)]
    pub fields: Vec<LaunchpadField>,
    #[serde(default)]
    pub hook_kind: Option<LaunchpadHookKind>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchpadAppDefaults {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub shard_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadDeploymentDefaults {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub replicas: u32,
    #[serde(default)]
    pub placement_preset: String,
    #[serde(default)]
    pub min_distinct_nodes: u32,
    #[serde(default)]
    pub node_selector: String,
}

impl Default for LaunchpadDeploymentDefaults {
    fn default() -> Self {
        Self {
            name: String::new(),
            slug: String::new(),
            port: 8080,
            replicas: 1,
            placement_preset: "single-node".to_string(),
            min_distinct_nodes: 1,
            node_selector: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadSourceDefaults {
    #[serde(default)]
    pub mode: LaunchpadSourceMode,
    #[serde(default)]
    pub image: String,
    #[serde(default = "default_tag")]
    pub tag: String,
    #[serde(default = "default_git_provider")]
    pub git_provider: String,
    #[serde(default)]
    pub git_repo_url: String,
    #[serde(default = "default_branch")]
    pub git_branch: String,
    #[serde(default = "default_git_auth_method")]
    pub git_auth_method: String,
    #[serde(default = "default_true")]
    pub auto_deploy: bool,
}

impl Default for LaunchpadSourceDefaults {
    fn default() -> Self {
        Self {
            mode: LaunchpadSourceMode::default(),
            image: String::new(),
            tag: default_tag(),
            git_provider: default_git_provider(),
            git_repo_url: String::new(),
            git_branch: default_branch(),
            git_auth_method: default_git_auth_method(),
            auto_deploy: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LaunchpadSourceMode {
    #[default]
    Image,
    Git,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchpadDomainDefaults {
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_path")]
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadStaticEnvVar {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadField {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub help_text: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub field_type: LaunchpadFieldType,
    #[serde(default)]
    pub options: Vec<LaunchpadFieldOption>,
    pub binding: LaunchpadFieldBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadFieldOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LaunchpadFieldType {
    #[default]
    Text,
    Secret,
    Number,
    Boolean,
    Select,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LaunchpadFieldBinding {
    AppName,
    AppSlug,
    AppShardMode,
    DeploymentName,
    DeploymentSlug,
    DeploymentPort,
    DeploymentReplicas,
    DeploymentPlacementPreset,
    DeploymentMinDistinctNodes,
    DeploymentNodeSelector,
    SourceMode,
    DeploymentImage,
    DeploymentTag,
    GitProvider,
    GitRepoUrl,
    GitBranch,
    GitAuthMethod,
    GitCredentials,
    DomainHost,
    DomainPath,
    EnvVar { key: String, secret: bool },
    PlatformSetting { key: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchpadHookKind {
    Oikonotes,
}

/// Request body for launchpad launch and source detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchpadLaunchRequest {
    #[serde(default)]
    pub values: HashMap<String, String>,
}

/// Response body from starting a launchpad-driven deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchpadLaunchResponse {
    pub launchpad_id: String,
    pub launchpad_version: u32,
    pub deployment_id: String,
    pub app_id: Option<String>,
    pub log_id: Option<String>,
    pub summary: String,
}

fn default_tag() -> String {
    "latest".to_string()
}

fn default_git_provider() -> String {
    "github".to_string()
}

fn default_git_auth_method() -> String {
    "none".to_string()
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_true() -> bool {
    true
}

fn default_path() -> String {
    "/".to_string()
}
