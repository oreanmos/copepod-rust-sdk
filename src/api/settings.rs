use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    // -- Platform settings --

    /// Get platform-wide settings (admin).
    pub async fn get_platform_settings(&self) -> Result<Value> {
        self.get("api/platform/settings").await
    }

    /// Update platform-wide settings (admin).
    pub async fn update_platform_settings(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put("api/platform/settings", body).await
    }

    // -- Per-app email sender --

    /// Get the email sender configuration for an app.
    pub async fn get_email_sender(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/email/sender",
            org_id, app_id
        ))
        .await
    }

    /// Create or update the email sender for an app.
    pub async fn upsert_email_sender(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!("api/platform/orgs/{}/apps/{}/email/sender", org_id, app_id),
            body,
        )
        .await
    }

    /// Delete the email sender for an app.
    pub async fn delete_email_sender(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/email/sender",
            org_id, app_id
        ))
        .await
    }

    // -- Per-app email templates --

    /// List all email templates for an app.
    pub async fn list_email_templates(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/email/templates",
            org_id, app_id
        ))
        .await
    }

    /// Get a specific email template by purpose.
    pub async fn get_email_template(
        &self,
        org_id: &str,
        app_id: &str,
        purpose: &str,
    ) -> Result<Value> {
        self.get(&format!(
            "api/platform/orgs/{}/apps/{}/email/templates/{}",
            org_id, app_id, purpose
        ))
        .await
    }

    /// Create or update an email template.
    pub async fn upsert_email_template(
        &self,
        org_id: &str,
        app_id: &str,
        purpose: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.put(
            &format!(
                "api/platform/orgs/{}/apps/{}/email/templates/{}",
                org_id, app_id, purpose
            ),
            body,
        )
        .await
    }

    /// Delete an email template.
    pub async fn delete_email_template(
        &self,
        org_id: &str,
        app_id: &str,
        purpose: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/apps/{}/email/templates/{}",
            org_id, app_id, purpose
        ))
        .await
    }

    /// Send a test email for an app.
    pub async fn send_test_email(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.post(
            &format!("api/platform/orgs/{}/apps/{}/email/test", org_id, app_id),
            body,
        )
        .await
    }
}
