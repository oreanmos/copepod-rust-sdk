//! Release payload of a deployment: what serves, what is rolling out, how the
//! last attempt ended. Served by `GET .../deployments/{id}/release`.

use serde::{Deserialize, Serialize};

/// Where a deployment's latest release attempt stands.
///
/// Phases the SDK does not know deserialise to [`ReleasePhase::Unknown`] so a
/// newer server never breaks an older client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleasePhase {
    /// No deploy has been attempted.
    Idle,
    /// An image is being built before anything is applied.
    Building,
    /// The new revision is applied and rolling out.
    RollingOut,
    /// The latest attempt finished and is serving.
    Succeeded,
    /// The latest attempt failed.
    Failed,
    /// The latest attempt found no room on the cluster and applied nothing.
    BlockedNoCapacity,
    /// A phase newer than this SDK.
    #[serde(other)]
    Unknown,
}

/// The release serving traffic now.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseServing {
    pub image: Option<String>,
    pub log_id: Option<String>,
    pub deployed_at: Option<String>,
    /// `None` when the cluster status was unreadable.
    #[serde(default)]
    pub ready: Option<u32>,
    /// `None` when the cluster status was unreadable.
    #[serde(default)]
    pub desired: Option<u32>,
}

/// The release the latest (or active) deploy action delivers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseTarget {
    pub log_id: String,
    /// `deploy`, `redeploy`, `start`, `stop` or `rollback`.
    pub action: String,
    pub requested_image: Option<String>,
    pub resolved_image: Option<String>,
    /// `surge`, `no_surge`, `outage` or `recreate`, once planned.
    pub plan: Option<String>,
    pub triggered_by: Option<String>,
}

/// One step of a deploy action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseStep {
    pub name: String,
    /// `pending`, `running`, `success`, `failed` or `skipped`.
    pub status: String,
    #[serde(default)]
    pub message: String,
    pub started: Option<String>,
    pub finished: Option<String>,
}

/// Rollout counters of the target revision plus its steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseProgress {
    pub updated: u32,
    pub updated_ready: u32,
    pub desired: u32,
    pub previous_serving: u32,
    pub current_step: Option<String>,
    #[serde(default)]
    pub steps: Vec<ReleaseStep>,
}

/// How the latest finished attempt ended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseOutcome {
    /// `success`, `failed` or `blocked`.
    pub status: String,
    /// Machine-readable reason such as `crash_loop` or `image_pull`.
    pub code: Option<String>,
    pub message: String,
}

fn runtime_available_default() -> bool {
    true
}

/// The release payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentRelease {
    pub phase: ReleasePhase,
    /// Opaque change token; compare for equality only.
    pub version: u64,
    pub configured_image: String,
    /// `false` when the cluster status could not be read; only the deploy-log
    /// fields are then meaningful.
    #[serde(default = "runtime_available_default")]
    pub runtime_available: bool,
    pub serving: Option<ReleaseServing>,
    pub target: Option<ReleaseTarget>,
    pub progress: Option<ReleaseProgress>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub outcome: Option<ReleaseOutcome>,
}

/// Rollout evaluation of the target revision, as published by the operator.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RolloutStatus {
    #[serde(default)]
    pub complete: bool,
    pub reason: Option<String>,
    pub since: Option<String>,
    pub message: Option<String>,
    #[serde(default)]
    pub desired: u32,
    #[serde(default)]
    pub updated: u32,
    #[serde(default)]
    pub updated_ready: u32,
    #[serde(default)]
    pub total: u32,
    pub target_pod_template_hash: Option<String>,
    pub target_digest: Option<String>,
    pub failing_pod: Option<String>,
    pub progress_deadline_seconds: Option<u32>,
}

/// Options for [`crate::CopepodClient::deploy_with_options`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeployOptions {
    /// Deploy mode; `None` sends `force`. The other server value is
    /// `if_image_changed`.
    pub mode: Option<String>,
    /// Accept a short outage when a single replica has no room to roll.
    pub allow_outage: bool,
}

/// Details of a `409 rollout_needs_outage` / `rollout_needs_capacity` error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolloutCapacityDetails {
    pub serving_image: Option<String>,
    pub needed: Option<ResourceAmount>,
    pub best_free: Option<ResourceAmount>,
    #[serde(default)]
    pub eligible_nodes: u32,
    #[serde(default)]
    pub outage_available: bool,
    pub startup_probe_seconds: Option<u32>,
}

/// CPU and memory amounts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceAmount {
    pub cpu_millicores: u64,
    pub memory_mib: u64,
}
