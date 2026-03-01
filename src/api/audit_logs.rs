use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    /// List audit log entries (admin only).
    ///
    /// Supports pagination via `page` and `per_page` query parameters.
    pub async fn list_audit_logs(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<Value> {
        let mut path = "api/platform/logs/audit".to_string();
        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        self.get(&path).await
    }
}
