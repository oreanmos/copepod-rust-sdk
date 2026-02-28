use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, Queue};

impl CopepodClient {
    /// List queues for an app.
    pub async fn list_queues(&self, app_id: &str) -> Result<ListResult<Queue>> {
        self.get(&format!("api/platform/apps/{}/queues", app_id))
            .await
    }

    /// Create a queue.
    pub async fn create_queue(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Queue> {
        self.post(&format!("api/platform/apps/{}/queues", app_id), body)
            .await
    }

    /// Delete a queue.
    pub async fn delete_queue(&self, app_id: &str, queue_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/apps/{}/queues/{}",
            app_id, queue_id
        ))
        .await
    }
}
