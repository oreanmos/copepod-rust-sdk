use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{Job, ListResult};

impl CopepodClient {
    /// List jobs for an app.
    pub async fn list_jobs(&self, app_id: &str) -> Result<ListResult<Job>> {
        self.get(&format!("api/platform/apps/{}/jobs", app_id))
            .await
    }

    /// Create a job.
    pub async fn create_job(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Job> {
        self.post(&format!("api/platform/apps/{}/jobs", app_id), body)
            .await
    }

    /// Get a specific job.
    pub async fn get_job(&self, app_id: &str, job_id: &str) -> Result<Job> {
        self.get(&format!("api/platform/apps/{}/jobs/{}", app_id, job_id))
            .await
    }

    /// Retry a failed job.
    pub async fn retry_job(
        &self,
        app_id: &str,
        job_id: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!("api/platform/apps/{}/jobs/{}/retry", app_id, job_id),
            &serde_json::json!({}),
        )
        .await
    }

    /// Cancel a pending or running job.
    pub async fn cancel_job(
        &self,
        app_id: &str,
        job_id: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!("api/platform/apps/{}/jobs/{}/cancel", app_id, job_id),
            &serde_json::json!({}),
        )
        .await
    }
}
