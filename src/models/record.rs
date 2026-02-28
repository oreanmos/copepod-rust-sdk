use serde::{Deserialize, Serialize};

/// A real-time record event received via SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEvent {
    pub action: String,
    pub collection: String,
    pub record: serde_json::Value,
}
