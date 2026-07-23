use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    EmailDeliveryEnqueueResponse, EmailDeliveryStatusBatchResponse, EmailDeliveryStatusResponse,
    TransactionalEmailRequest,
};

/// Transactional email helpers bound to an organization and application.
#[derive(Debug, Clone)]
pub struct ScopedEmailClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedEmailClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: &str, app_id: &str) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// Durably enqueue a transactional email.
    pub async fn enqueue(
        &self,
        idempotency_key: &str,
        request: &TransactionalEmailRequest,
    ) -> Result<EmailDeliveryEnqueueResponse> {
        self.client
            .enqueue_transactional_email(&self.org_id, &self.app_id, idempotency_key, request)
            .await
    }

    /// Fetch one delivery state.
    pub async fn get(&self, delivery_id: &str) -> Result<EmailDeliveryStatusResponse> {
        self.client
            .get_email_delivery(&self.org_id, &self.app_id, delivery_id)
            .await
    }

    /// Fetch between 1 and 100 delivery states.
    pub async fn statuses(
        &self,
        delivery_ids: &[String],
    ) -> Result<EmailDeliveryStatusBatchResponse> {
        self.client
            .get_email_delivery_statuses(&self.org_id, &self.app_id, delivery_ids)
            .await
    }

    /// Retry a terminally failed delivery.
    pub async fn retry(&self, delivery_id: &str) -> Result<EmailDeliveryStatusResponse> {
        self.client
            .retry_email_delivery(&self.org_id, &self.app_id, delivery_id)
            .await
    }

    /// Cancel a pending or failed delivery.
    pub async fn cancel(&self, delivery_id: &str) -> Result<EmailDeliveryStatusResponse> {
        self.client
            .cancel_email_delivery(&self.org_id, &self.app_id, delivery_id)
            .await
    }
}
