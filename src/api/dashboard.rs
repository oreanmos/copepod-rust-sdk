use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{DashboardStats, GraphData, ResourceInfo};

impl CopepodClient {
    /// Get high-level dashboard statistics.
    pub async fn get_dashboard_stats(&self) -> Result<DashboardStats> {
        self.get("api/dashboard/stats").await
    }

    /// Get dashboard graph data for the given number of hours.
    pub async fn get_dashboard_graphs(&self, hours: u32) -> Result<Vec<GraphData>> {
        self.get(&format!("api/dashboard/graphs?hours={}", hours))
            .await
    }

    /// Get server metrics for the given number of hours.
    pub async fn get_server_metrics(&self, hours: u32) -> Result<Vec<GraphData>> {
        self.get(&format!("api/dashboard/server-metrics?hours={}", hours))
            .await
    }

    /// Get server resource usage information.
    pub async fn get_resources(&self) -> Result<ResourceInfo> {
        self.get("api/dashboard/resources").await
    }
}
