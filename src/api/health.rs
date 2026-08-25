use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::ServingHealthResponse;

impl CopepodClient {
    /// Check whether consistency-sensitive traffic can reach a serving leader.
    ///
    /// In Raft mode an election or unavailable quorum returns a
    /// [`crate::CopepodError`] for which
    /// [`crate::CopepodError::is_raft_leader_unavailable`] is true. The SDK
    /// leaves retry decisions to the caller so mutations are never replayed
    /// implicitly.
    pub async fn health_serving(&self) -> Result<ServingHealthResponse> {
        self.get_public("api/platform/health/serving").await
    }
}
