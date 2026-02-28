use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::ActionEntry;

impl CopepodClient {
    /// List all available actions.
    pub async fn list_actions(&self) -> Result<Vec<ActionEntry>> {
        self.get("api/actions").await
    }

    /// Get the action permission matrix for an app.
    pub async fn get_action_matrix(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/orgs/{}/apps/{}/actions/matrix",
            org_id, app_id
        ))
        .await
    }

    /// Update an action-role mapping in the permission matrix.
    pub async fn update_action_matrix(
        &self,
        org_id: &str,
        app_id: &str,
        action_key: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!(
                "api/orgs/{}/apps/{}/actions/matrix/{}",
                org_id, app_id, action_key
            ),
            body,
        )
        .await
    }

    /// Synchronize actions with the server.
    pub async fn sync_actions(&self) -> Result<()> {
        self.post_empty("api/actions/sync", &serde_json::json!({}))
            .await
    }
}
