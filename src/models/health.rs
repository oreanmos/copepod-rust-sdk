use serde::{Deserialize, Serialize};

/// Response from the leader-fenced serving health endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServingHealthResponse {
    pub status: String,
}
