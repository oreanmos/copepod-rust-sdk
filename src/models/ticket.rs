use serde::{Deserialize, Serialize};

/// A support ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub ticket_number: String,
    pub app_id: String,
    #[serde(default)]
    pub app_name: Option<String>,
    #[serde(default)]
    pub app_slug: Option<String>,
    pub org_id: String,
    pub user_id: String,
    pub user_email: String,
    pub user_name: String,
    pub subject: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub priority: String,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub error_details: Option<String>,
    #[serde(default)]
    pub error_url: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub context: serde_json::Value,
    #[serde(default)]
    pub assigned_to_id: Option<String>,
    #[serde(default)]
    pub assigned_to_name: Option<String>,
    #[serde(default)]
    pub internal_notes: Option<String>,
    #[serde(default)]
    pub first_response_at: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub closed_at: Option<String>,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub last_comment_at: Option<String>,
    pub created: String,
    pub updated: String,
}

/// Request body to create a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCreate {
    #[serde(default)]
    pub id: Option<String>,
    pub subject: String,
    pub description: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub error_details: Option<String>,
    #[serde(default)]
    pub error_url: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub context: serde_json::Value,
}

fn default_category() -> String {
    "question".to_string()
}

fn default_priority() -> String {
    "normal".to_string()
}

/// Request body to update a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketUpdate {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub assigned_to_id: Option<String>,
    #[serde(default)]
    pub assigned_to_name: Option<String>,
    #[serde(default)]
    pub internal_notes: Option<String>,
}

/// A comment on a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketComment {
    pub id: String,
    pub ticket_id: String,
    pub user_id: String,
    pub user_name: String,
    pub content: String,
    pub is_from_support: bool,
    pub is_internal: bool,
    pub created: String,
}

/// Request body to create a ticket comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketCommentCreate {
    pub content: String,
    #[serde(default)]
    pub is_from_support: bool,
    #[serde(default)]
    pub is_internal: bool,
}

/// An attachment on a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketAttachment {
    pub id: String,
    #[serde(default)]
    pub ticket_id: Option<String>,
    #[serde(default)]
    pub comment_id: Option<String>,
    pub file_name: String,
    pub stored_key: String,
    pub content_type: String,
    pub file_size: u64,
    pub uploaded_by: String,
    pub created: String,
}

/// Aggregate ticket statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketStats {
    pub total: u64,
    pub open: u64,
    pub in_progress: u64,
    pub awaiting_user: u64,
    pub awaiting_support: u64,
    pub resolved: u64,
    pub closed: u64,
    pub avg_resolution_hours: f64,
}

/// Query parameters for listing tickets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketListQuery {
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub per_page: Option<u32>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub assigned_to_id: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
}
