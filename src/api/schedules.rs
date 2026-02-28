use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, Schedule};

impl CopepodClient {
    /// List schedules for an app.
    pub async fn list_schedules(&self, app_id: &str) -> Result<ListResult<Schedule>> {
        self.get(&format!("api/platform/apps/{}/schedules", app_id))
            .await
    }

    /// Create a schedule.
    pub async fn create_schedule(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Schedule> {
        self.post(&format!("api/platform/apps/{}/schedules", app_id), body)
            .await
    }

    /// Update a schedule.
    pub async fn update_schedule(
        &self,
        app_id: &str,
        schedule_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Schedule> {
        self.patch(
            &format!(
                "api/platform/apps/{}/schedules/{}",
                app_id, schedule_id
            ),
            body,
        )
        .await
    }

    /// Delete a schedule.
    pub async fn delete_schedule(&self, app_id: &str, schedule_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/apps/{}/schedules/{}",
            app_id, schedule_id
        ))
        .await
    }
}
