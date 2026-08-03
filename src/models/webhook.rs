use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundWebhook {
    pub id: String,
    pub app_id: String,
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    pub active: bool,
    pub description: String,
    pub created: String,
    pub updated: String,
}

/// A newly created outbound webhook and its signing secret.
///
/// `secret` is disclosed exactly once and is absent from list and update
/// responses.
#[derive(Clone, Serialize, Deserialize)]
pub struct OutboundWebhookCreated {
    #[serde(flatten)]
    pub webhook: OutboundWebhook,
    pub secret: String,
}

impl std::fmt::Debug for OutboundWebhookCreated {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OutboundWebhookCreated")
            .field("webhook", &self.webhook)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutboundWebhookCreate {
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutboundWebhookUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event_type: String,
    pub payload: String,
    pub status: String,
    #[serde(default)]
    pub response_code: Option<i32>,
    #[serde(default)]
    pub response_body: Option<String>,
    pub attempts: i32,
    #[serde(default)]
    pub next_retry: Option<String>,
    pub created: String,
}

/// Result of queuing a test delivery for an outbound webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundWebhookTestResponse {
    pub webhook_id: String,
    pub url: String,
    pub delivery: WebhookDelivery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub id: String,
    pub app_id: String,
    pub event_pattern: String,
    pub handler_type: String,
    pub handler_config: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscriptionCreate {
    pub event_pattern: String,
    pub handler_type: String,
    #[serde(default)]
    pub handler_config: String,
}

#[cfg(test)]
mod tests {
    use super::{OutboundWebhook, OutboundWebhookCreated};

    fn webhook_json() -> serde_json::Value {
        serde_json::json!({
            "id": "wh-1",
            "app_id": "app-1",
            "url": "https://hooks.example.test/copepod",
            "events": ["records.created"],
            "active": true,
            "description": "record sink",
            "created": "2026-07-22T00:00:00Z",
            "updated": "2026-07-22T00:00:00Z"
        })
    }

    #[test]
    fn ordinary_webhook_responses_do_not_model_the_secret() {
        let webhook: OutboundWebhook = serde_json::from_value(webhook_json()).unwrap();
        let value = serde_json::to_value(webhook).unwrap();
        assert!(value.get("secret").is_none());
    }

    #[test]
    fn create_response_requires_the_one_time_secret() {
        assert!(serde_json::from_value::<OutboundWebhookCreated>(webhook_json()).is_err());

        let mut value = webhook_json();
        value["secret"] = serde_json::Value::String("a".repeat(64));
        let created: OutboundWebhookCreated = serde_json::from_value(value).unwrap();
        assert_eq!(created.secret.len(), 64);
        let debug = format!("{created:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(&"a".repeat(64)));
    }
}
