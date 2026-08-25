use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role reported for a member of the control-plane Raft cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RaftMemberRole {
    /// The current Raft leader.
    Leader,
    /// A voting follower.
    Voter,
    /// A non-voting member that must catch up before promotion.
    Learner,
}

/// One member in the committed control-plane Raft membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaftMemberInfo {
    /// Stable Raft node ID.
    pub node_id: u64,
    /// Raft RPC address in `host:port` form.
    pub address: String,
    /// Current membership role.
    pub role: RaftMemberRole,
    /// Last log index the leader reports replicated to this member.
    pub matched_index: Option<u64>,
    /// Whether the member has replicated through the leader's applied index.
    pub caught_up: bool,
}

/// Leader-confirmed control-plane Raft membership and replication progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaftMembershipResponse {
    /// Log index containing the committed membership.
    pub membership_log_index: Option<u64>,
    /// Current leader node ID.
    pub leader_node_id: u64,
    /// Last index applied by the leader state machine.
    pub leader_last_applied_index: Option<u64>,
    /// Whether the membership is still in joint consensus.
    pub joint: bool,
    /// Voting members, including the leader.
    pub voters: Vec<RaftMemberInfo>,
    /// Non-voting learners.
    pub learners: Vec<RaftMemberInfo>,
}

impl RaftMembershipResponse {
    /// Return the exact voter IDs to use as a removal precondition.
    pub fn voter_ids(&self) -> Vec<u64> {
        self.voters.iter().map(|member| member.node_id).collect()
    }
}

/// Request to add a new non-voting Raft learner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddRaftLearnerRequest {
    /// Stable node ID that is not already in membership.
    pub node_id: u64,
    /// Raft RPC address in `host:port` form.
    pub address: String,
    /// Membership log index from the immediately preceding membership read.
    pub expected_membership_log_index: u64,
}

/// Optimistic-concurrency precondition for learner promotion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoteRaftLearnerRequest {
    /// Membership log index from the immediately preceding membership read.
    pub expected_membership_log_index: u64,
}

/// Preconditions and destructive confirmation for removing a Raft member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoveRaftMemberRequest {
    /// Membership log index from the immediately preceding membership read.
    pub expected_membership_log_index: u64,
    /// Exact voter IDs from the immediately preceding membership read.
    pub expected_voters: Vec<u64>,
    /// Permit an incident-only transition from three voters to two.
    #[serde(default)]
    pub allow_temporary_two_voters: bool,
    /// Exact destructive confirmation required by the server.
    pub confirmation: String,
}

impl RemoveRaftMemberRequest {
    /// Build the normal removal request for a learner or a voter with a replacement.
    pub fn confirmed(
        node_id: u64,
        expected_membership_log_index: u64,
        expected_voters: Vec<u64>,
    ) -> Self {
        Self {
            expected_membership_log_index,
            expected_voters,
            allow_temporary_two_voters: false,
            confirmation: format!("REMOVE RAFT NODE {node_id}"),
        }
    }

    /// Build the incident-only request that temporarily leaves two voters.
    ///
    /// A two-voter cluster has no failure tolerance. Add and promote the
    /// replacement learner immediately after removing the stale voter.
    pub fn temporary_two_voter_repair(
        node_id: u64,
        expected_membership_log_index: u64,
        expected_voters: Vec<u64>,
    ) -> Self {
        Self {
            expected_membership_log_index,
            expected_voters,
            allow_temporary_two_voters: true,
            confirmation: format!("REMOVE RAFT VOTER {node_id} LEAVING TWO VOTERS"),
        }
    }
}

/// A Raft shard group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardGroup {
    pub id: String,
    pub name: String,
    pub group_number: u64,
    pub replicas: u32,
    pub leader_node_id: Option<u64>,
    pub status: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Request to create a production Raft shard group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShardGroupRequest {
    pub name: String,
    pub group_number: u64,
    pub replicas: u32,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

/// Request an epoch-guarded shard assignment cutover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveShardRequest {
    pub target_group_id: String,
    /// Assignment epoch observed when the caller read the shard.
    ///
    /// The SDK always requires this value so stale and ABA cutovers are
    /// rejected instead of silently overwriting a newer assignment.
    pub expected_assignment_epoch: u64,
}

/// Durable shard move job returned when a move is queued or inspected.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShardMoveJob {
    pub move_id: String,
    pub shard_id: String,
    /// Legacy completed-response target identifier. Queued jobs expose the
    /// stable numeric `target_group` while this field remains empty.
    pub target_group_id: String,
    /// Legacy completed-response epoch. Queued jobs use
    /// `expected_assignment_epoch` until cutover completes.
    pub assignment_epoch: u64,
    pub source_group: u64,
    pub target_group: u64,
    pub expected_assignment_epoch: u64,
    pub phase: String,
    pub status: String,
    pub message: String,
    pub snapshot_sha256: Option<String>,
    pub snapshot_bytes: Option<u64>,
    pub source_barrier_log_index: Option<u64>,
    pub target_install_log_index: Option<u64>,
    pub started_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    #[serde(default)]
    pub attempts: u32,
    pub last_error: Option<String>,
}

/// Backward-compatible name for the durable move response.
pub type MoveShardResponse = ShardMoveJob;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMovePage {
    pub moves: Vec<ShardMoveJob>,
    pub limit: u32,
    pub offset: u32,
    pub next_offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceSkippedMove {
    pub shard_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceExecutionResponse {
    pub accepted: usize,
    pub move_ids: Vec<String>,
    pub skipped: Vec<RebalanceSkippedMove>,
    pub message: String,
}

/// Request to manually register a shard database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShardRequest {
    pub app_id: String,
    pub shard_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub db_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shard_group_id: Option<String>,
}

/// Response for manual shard creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardResponse {
    pub id: String,
    pub app_id: String,
    pub shard_key: String,
    pub db_path: String,
    pub shard_group_id: Option<String>,
    pub assignment_epoch: u64,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::{MoveShardRequest, RemoveRaftMemberRequest};

    #[test]
    fn shard_move_always_serializes_the_cas_epoch() {
        let value = serde_json::to_value(MoveShardRequest {
            target_group_id: "group-b".to_string(),
            expected_assignment_epoch: 7,
        })
        .unwrap();

        assert_eq!(value["target_group_id"], "group-b");
        assert_eq!(value["expected_assignment_epoch"], 7);
    }

    #[test]
    fn raft_removal_helpers_generate_exact_confirmations() {
        let normal = RemoveRaftMemberRequest::confirmed(4, 21, vec![1, 2, 3, 4]);
        assert!(!normal.allow_temporary_two_voters);
        assert_eq!(normal.confirmation, "REMOVE RAFT NODE 4");

        let incident = RemoveRaftMemberRequest::temporary_two_voter_repair(3, 22, vec![1, 2, 3]);
        assert!(incident.allow_temporary_two_voters);
        assert_eq!(
            incident.confirmation,
            "REMOVE RAFT VOTER 3 LEAVING TWO VOTERS"
        );
    }
}
