use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AddRaftLearnerRequest, CreateShardGroupRequest, CreateShardRequest, MoveShardRequest,
    MoveShardResponse, PromoteRaftLearnerRequest, RaftMembershipResponse, RemoveRaftMemberRequest,
    ShardGroup, ShardResponse,
};

impl CopepodClient {
    /// Get a leader-confirmed view of the control-plane Raft membership.
    pub async fn get_raft_membership(&self) -> Result<RaftMembershipResponse> {
        self.get("api/platform/cluster/raft/membership").await
    }

    /// Add a node as a non-voting control-plane Raft learner.
    ///
    /// Use the membership log index from an immediately preceding
    /// [`Self::get_raft_membership`] call. Promotion is a separate operation.
    pub async fn add_raft_learner(
        &self,
        body: &AddRaftLearnerRequest,
    ) -> Result<RaftMembershipResponse> {
        self.post("api/platform/cluster/raft/learners", body).await
    }

    /// Promote a caught-up control-plane Raft learner to voter.
    ///
    /// The server rejects stale membership indexes and learners whose matched
    /// index has not reached the leader's applied index.
    pub async fn promote_raft_learner(
        &self,
        node_id: u64,
        body: &PromoteRaftLearnerRequest,
    ) -> Result<RaftMembershipResponse> {
        self.post(
            &format!("api/platform/cluster/raft/learners/{node_id}/promote"),
            body,
        )
        .await
    }

    /// Remove a non-leader control-plane Raft member through joint consensus.
    ///
    /// Build the request with [`RemoveRaftMemberRequest::confirmed`] for the
    /// normal replacement-first workflow. The incident-only two-voter helper
    /// must be followed immediately by adding and promoting a replacement.
    pub async fn remove_raft_member(
        &self,
        node_id: u64,
        body: &RemoveRaftMemberRequest,
    ) -> Result<RaftMembershipResponse> {
        self.post(
            &format!("api/platform/cluster/raft/members/{node_id}/remove"),
            body,
        )
        .await
    }

    /// List all shard groups in the cluster.
    pub async fn list_shard_groups(&self) -> Result<Vec<ShardGroup>> {
        self.get("api/platform/cluster/shard-groups").await
    }

    /// Create a new shard group.
    pub async fn create_shard_group(&self, body: &CreateShardGroupRequest) -> Result<ShardGroup> {
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

    /// Move a shard with an optimistic-concurrency assignment epoch.
    ///
    /// Set `expected_assignment_epoch` from the latest shard response to make
    /// stale and ABA cutovers fail with HTTP 409 instead of changing routing.
    pub async fn move_shard(
        &self,
        shard_id: &str,
        body: &MoveShardRequest,
    ) -> Result<MoveShardResponse> {
        self.post(
            &format!("api/platform/cluster/shards/{}/move", shard_id),
            body,
        )
        .await
    }

    /// Compatibility alias for [`Self::move_shard`].
    pub async fn move_shard_checked(
        &self,
        shard_id: &str,
        body: &MoveShardRequest,
    ) -> Result<MoveShardResponse> {
        self.move_shard(shard_id, body).await
    }

    /// Manually create/register a shard.
    pub async fn create_shard(&self, body: &CreateShardRequest) -> Result<ShardResponse> {
        self.post("api/platform/cluster/shards", body).await
    }
}
