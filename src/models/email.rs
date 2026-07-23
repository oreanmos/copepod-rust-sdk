use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Maximum number of delivery IDs accepted by the batch status endpoint.
pub const MAX_EMAIL_DELIVERY_STATUS_BATCH: usize = 100;

/// Lifecycle state of a queued transactional email delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailDeliveryState {
    Pending,
    Sending,
    Sent,
    Failed,
    Cancelled,
}

/// Request body for enqueuing a transactional email.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionalEmailRequest {
    pub to: String,
    pub subject: String,
    pub body_html: String,
}

/// Result returned after a transactional email has been durably enqueued.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailDeliveryEnqueueResponse {
    pub delivery_id: String,
    pub status: EmailDeliveryState,
    pub deduplicated: bool,
}

/// Public delivery state. Message content and recipient data are never exposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailDeliveryStatusResponse {
    pub delivery_id: String,
    pub status: EmailDeliveryState,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub sent_at: Option<DateTime<Utc>>,
    /// Sanitized by the server and safe to show to application operators.
    #[serde(default)]
    pub last_error: Option<String>,
}

/// Request body for fetching a bounded batch of delivery states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailDeliveryStatusBatchRequest {
    pub delivery_ids: Vec<String>,
}

/// Response for a bounded batch of delivery states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailDeliveryStatusBatchResponse {
    pub deliveries: Vec<EmailDeliveryStatusResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_status_uses_server_wire_values() {
        for (status, expected) in [
            (EmailDeliveryState::Pending, "\"pending\""),
            (EmailDeliveryState::Sending, "\"sending\""),
            (EmailDeliveryState::Sent, "\"sent\""),
            (EmailDeliveryState::Failed, "\"failed\""),
            (EmailDeliveryState::Cancelled, "\"cancelled\""),
        ] {
            assert_eq!(serde_json::to_string(&status).unwrap(), expected);
        }
    }

    #[test]
    fn delivery_response_does_not_model_message_content() {
        let delivery: EmailDeliveryStatusResponse = serde_json::from_value(serde_json::json!({
            "delivery_id": "delivery_1",
            "status": "failed",
            "attempts": 3,
            "created_at": "2026-07-22T12:00:00Z",
            "updated_at": "2026-07-22T12:05:00Z",
            "sent_at": null,
            "last_error": "provider temporarily unavailable"
        }))
        .unwrap();

        assert_eq!(delivery.status, EmailDeliveryState::Failed);
        assert_eq!(delivery.attempts, 3);
        assert_eq!(
            delivery.last_error.as_deref(),
            Some("provider temporarily unavailable")
        );
    }
}
