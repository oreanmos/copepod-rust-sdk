use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ShardGroup, ShardResponse};

impl CopepodClient {
    /// List all shard groups in the cluster.
    pub async fn list_shard_groups(&self) -> Result<Vec<ShardGroup>> {
        self.get("api/platform/cluster/shard-groups").await
    }

    /// Create a new shard group.
    pub async fn create_shard_group(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<ShardGroup> {
        self.post("api/platform/cluster/shard-groups", body).await
    }

    /// Get a shard group by ID.
    pub async fn get_shard_group(&self, id: &str) -> Result<ShardGroup> {
        self.get(&format!("api/platform/cluster/shard-groups/{}", id))
            .await
    }

    /// Delete a shard group.
    pub async fn delete_shard_group(&self, id: &str) -> Result<()> {
        self.delete(&format!("api/platform/cluster/shard-groups/{}", id))
            .await
    }

    /// Move a shard to a different group.
    pub async fn move_shard(
        &self,
        shard_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.post(
            &format!("api/platform/cluster/shards/{}/move", shard_id),
            body,
        )
        .await
    }

    /// Manually create/register a shard.
    pub async fn create_shard(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<ShardResponse> {
        self.post("api/platform/cluster/shards", body).await
    }
}
