use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ActionLog, ListResult, LogStats};

impl CopepodClient {
    /// List action logs (with optional query parameters).
    pub async fn list_logs(&self) -> Result<ListResult<ActionLog>> {
        self.get("api/logs").await
    }

    /// Get a single log entry by ID.
    pub async fn get_log(&self, id: &str) -> Result<ActionLog> {
        self.get(&format!("api/logs/{}", id)).await
    }

    /// Get aggregated log statistics.
    pub async fn get_log_stats(&self) -> Result<LogStats> {
        self.get("api/logs/stats").await
    }

    /// Clean up logs older than the given number of days.
    pub async fn cleanup_logs(&self, days: u32) -> Result<Value> {
        let body = serde_json::json!({ "days": days });
        self.post("api/logs/cleanup", &body).await
    }
}
