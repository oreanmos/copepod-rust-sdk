use serde::{Deserialize, Serialize};

/// High-level dashboard statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    #[serde(default)]
    pub total_users: Option<u64>,
    #[serde(default)]
    pub total_apps: Option<u64>,
    #[serde(default)]
    pub total_orgs: Option<u64>,
    #[serde(default)]
    pub total_records: Option<u64>,
    #[serde(default)]
    pub storage_used: Option<u64>,
}

/// A single metrics data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsPoint {
    pub timestamp: String,
    pub value: f64,
    #[serde(default)]
    pub label: Option<String>,
}

/// Graph data for dashboard charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub label: String,
    pub points: Vec<MetricsPoint>,
}

/// Server resource information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    #[serde(default)]
    pub cpu_usage: Option<f64>,
    #[serde(default)]
    pub memory_usage: Option<f64>,
    #[serde(default)]
    pub disk_usage: Option<DiskUsage>,
}

/// Disk usage details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}
