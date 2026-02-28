use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, MediaJob};

impl CopepodClient {
    /// List media jobs for an app.
    pub async fn list_media_jobs(&self, app_id: &str) -> Result<ListResult<MediaJob>> {
        self.get(&format!("api/platform/apps/{}/media/jobs", app_id))
            .await
    }

    /// Create a media processing job.
    pub async fn create_media_job(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<MediaJob> {
        self.post(
            &format!("api/platform/apps/{}/media/jobs", app_id),
            body,
        )
        .await
    }

    /// Get a specific media job.
    pub async fn get_media_job(&self, app_id: &str, job_id: &str) -> Result<MediaJob> {
        self.get(&format!(
            "api/platform/apps/{}/media/jobs/{}",
            app_id, job_id
        ))
        .await
    }
}
