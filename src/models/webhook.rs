use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundWebhook {
    pub id: String,
    pub app_id: String,
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: Option<String>,
    pub active: bool,
    #[serde(default)]
    pub description: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundWebhookCreate {
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundWebhookUpdate {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub events: Option<Vec<String>>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event_type: String,
    #[serde(default)]
    pub payload: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub id: String,
    pub app_id: String,
    pub event_pattern: String,
    pub handler_type: String,
    #[serde(default)]
    pub handler_config: Option<String>,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscriptionCreate {
    pub event_pattern: String,
    pub handler_type: String,
    #[serde(default)]
    pub handler_config: Option<String>,
}
