use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{EventSubscription, ListResult, OutboundWebhook, WebhookDelivery};

impl CopepodClient {
    /// List outbound webhooks for an app.
    pub async fn list_webhooks(&self, app_id: &str) -> Result<ListResult<OutboundWebhook>> {
        self.get(&format!("api/platform/apps/{}/webhooks", app_id))
            .await
    }

    /// Create an outbound webhook.
    pub async fn create_webhook(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<OutboundWebhook> {
        self.post(&format!("api/platform/apps/{}/webhooks", app_id), body)
            .await
    }

    /// Update an outbound webhook.
    pub async fn update_webhook(
        &self,
        app_id: &str,
        webhook_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<OutboundWebhook> {
        self.patch(
            &format!("api/platform/apps/{}/webhooks/{}", app_id, webhook_id),
            body,
        )
        .await
    }

    /// Delete an outbound webhook.
    pub async fn delete_webhook(&self, app_id: &str, webhook_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/apps/{}/webhooks/{}",
            app_id, webhook_id
        ))
        .await
    }

    /// Test a webhook by sending a test payload.
    pub async fn test_webhook(
        &self,
        app_id: &str,
        webhook_id: &str,
    ) -> Result<serde_json::Value> {
        self.post(
            &format!(
                "api/platform/apps/{}/webhooks/{}/test",
                app_id, webhook_id
            ),
            &serde_json::json!({}),
        )
        .await
    }

    /// List recent deliveries for a webhook.
    pub async fn list_deliveries(
        &self,
        app_id: &str,
        webhook_id: &str,
    ) -> Result<ListResult<WebhookDelivery>> {
        self.get(&format!(
            "api/platform/apps/{}/webhooks/{}/deliveries",
            app_id, webhook_id
        ))
        .await
    }

    /// List event subscriptions for an app.
    pub async fn list_event_subscriptions(
        &self,
        app_id: &str,
    ) -> Result<ListResult<EventSubscription>> {
        self.get(&format!(
            "api/platform/apps/{}/events/subscriptions",
            app_id
        ))
        .await
    }

    /// Create an event subscription.
    pub async fn create_event_subscription(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<EventSubscription> {
        self.post(
            &format!("api/platform/apps/{}/events/subscriptions", app_id),
            body,
        )
        .await
    }

    /// Delete an event subscription.
    pub async fn delete_event_subscription(
        &self,
        app_id: &str,
        sub_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/apps/{}/events/subscriptions/{}",
            app_id, sub_id
        ))
        .await
    }
}
