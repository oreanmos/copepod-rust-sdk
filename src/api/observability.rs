use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{Alert, ErrorGroup, ListResult};

impl CopepodClient {
    /// List error groups for an app.
    pub async fn list_errors(&self, app_id: &str) -> Result<ListResult<ErrorGroup>> {
        self.get(&format!("api/platform/apps/{}/errors", app_id))
            .await
    }

    /// Get a specific error group.
    pub async fn get_error(&self, app_id: &str, group_id: &str) -> Result<ErrorGroup> {
        self.get(&format!(
            "api/platform/apps/{}/errors/{}",
            app_id, group_id
        ))
        .await
    }

    /// Resolve an error group.
    pub async fn resolve_error(
        &self,
        app_id: &str,
        group_id: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!(
                "api/platform/apps/{}/errors/{}/resolve",
                app_id, group_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// List alert rules for an app.
    pub async fn list_alerts(&self, app_id: &str) -> Result<ListResult<Alert>> {
        self.get(&format!("api/platform/apps/{}/alerts", app_id))
            .await
    }

    /// Create an alert rule.
    pub async fn create_alert(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Alert> {
        self.post(&format!("api/platform/apps/{}/alerts", app_id), body)
            .await
    }

    /// Update an alert rule.
    pub async fn update_alert(
        &self,
        app_id: &str,
        alert_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Alert> {
        self.patch(
            &format!("api/platform/apps/{}/alerts/{}", app_id, alert_id),
            body,
        )
        .await
    }

    /// Delete an alert rule.
    pub async fn delete_alert(&self, app_id: &str, alert_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/apps/{}/alerts/{}",
            app_id, alert_id
        ))
        .await
    }
}
