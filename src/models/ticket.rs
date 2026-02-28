use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A support ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    pub status: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Request body to create a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCreate {
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
}

/// Request body to update a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketUpdate {
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
}

/// A comment on a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketComment {
    pub id: String,
    pub ticket_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    pub body: String,
    pub created: DateTime<Utc>,
}

/// Request body to create a ticket comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCommentCreate {
    pub body: String,
}

/// An attachment on a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketAttachment {
    pub id: String,
    pub ticket_id: String,
    pub filename: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    pub created: DateTime<Utc>,
}
