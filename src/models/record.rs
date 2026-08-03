use serde::{Deserialize, Serialize};

/// Record mutation types that can be selected for a real-time subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordEventAction {
    Create,
    Update,
    Delete,
}

impl RecordEventAction {
    /// Wire value accepted by the realtime `actions` filter.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}

/// Optional filters and replay cursor for a real-time record subscription.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RealtimeSubscriptionOptions {
    /// Subscribe only to these collection names. An empty list subscribes to
    /// every collection the authenticated caller is allowed to list.
    pub collections: Vec<String>,
    /// Subscribe only to these mutation types. An empty list includes all
    /// mutation types.
    pub actions: Vec<RecordEventAction>,
    /// Resume after this event identifier when the server still retains it.
    pub last_event_id: Option<u64>,
}

/// A real-time record event received via SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEvent {
    /// Monotonic event cursor used with
    /// [`RealtimeSubscriptionOptions::last_event_id`].
    pub id: u64,
    pub action: String,
    pub collection: String,
    pub record: serde_json::Value,
}
