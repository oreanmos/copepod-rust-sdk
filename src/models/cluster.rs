use serde::{Deserialize, Serialize};

/// A Raft shard group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardGroup {
    pub id: String,
    pub name: String,
    pub group_number: u64,
    pub replicas: u32,
    pub leader_node_id: Option<u64>,
    pub status: String,
}

/// Response for manual shard creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardResponse {
    pub id: String,
    pub app_id: String,
    pub shard_key: String,
    pub db_path: String,
    pub shard_group_id: Option<String>,
    pub status: String,
}
