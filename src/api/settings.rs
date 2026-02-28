use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    /// Get email settings for an app.
    pub async fn get_email_settings(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/orgs/{}/apps/{}/settings/email",
            org_id, app_id
        ))
        .await
    }

    /// Update email settings for an app.
    pub async fn update_email_settings(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!("api/orgs/{}/apps/{}/settings/email", org_id, app_id),
            body,
        )
        .await
    }

    /// Get auth settings for an app.
    pub async fn get_auth_settings(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/orgs/{}/apps/{}/settings/auth",
            org_id, app_id
        ))
        .await
    }

    /// Update auth settings for an app.
    pub async fn update_auth_settings(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!("api/orgs/{}/apps/{}/settings/auth", org_id, app_id),
            body,
        )
        .await
    }

    /// Get platform-wide settings (admin).
    pub async fn get_platform_settings(&self) -> Result<Value> {
        self.get("api/settings").await
    }

    /// Update platform-wide settings (admin).
    pub async fn update_platform_settings(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put("api/settings", body).await
    }
}
