use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, User};

impl CopepodClient {
    /// List users of an app.
    pub async fn list_app_users(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ListResult<User>> {
        self.get(&format!("api/platform/orgs/{}/apps/{}/users", org_id, app_id))
            .await
    }

    /// Get an app user by ID.
    pub async fn get_app_user(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
    ) -> Result<User> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/users/{}",
            org_id, app_id, user_id
        ))
        .await
    }

    /// Get statistics for an app user.
    pub async fn get_user_stats(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/users/{}/stats",
            org_id, app_id, user_id
        ))
        .await
    }

    /// Get achievements for an app user.
    pub async fn get_user_achievements(
        &self,
        org_id: &str,
        app_id: &str,
        user_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/users/{}/achievements",
            org_id, app_id, user_id
        ))
        .await
    }

    /// List all achievements for an app.
    pub async fn list_achievements(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Vec<Value>> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/achievements",
            org_id, app_id
        ))
        .await
    }

    /// Create an achievement.
    pub async fn create_achievement(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.post(
            &format!("api/platform/orgs/{}/apps/{}/achievements", org_id, app_id),
            body,
        )
        .await
    }

    /// Update an achievement.
    pub async fn update_achievement(
        &self,
        org_id: &str,
        app_id: &str,
        achievement_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.patch(
            &format!(
                "api/platform/orgs/{}/apps/{}/achievements/{}",
                org_id, app_id, achievement_id
            ),
            body,
        )
        .await
    }

    /// Delete an achievement.
    pub async fn delete_achievement(
        &self,
        org_id: &str,
        app_id: &str,
        achievement_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/achievements/{}",
            org_id, app_id, achievement_id
        ))
        .await
    }
}
