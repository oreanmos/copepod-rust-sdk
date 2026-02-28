use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{Bucket, ListResult};

impl CopepodClient {
    /// List buckets for an organization.
    pub async fn list_buckets(&self, org_id: &str) -> Result<ListResult<Bucket>> {
        self.get(&format!("api/platform/orgs/{}/buckets", org_id))
            .await
    }

    /// Create a bucket.
    pub async fn create_bucket(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Bucket> {
        self.post(&format!("api/platform/orgs/{}/buckets", org_id), body)
            .await
    }

    /// Update a bucket.
    pub async fn update_bucket(
        &self,
        org_id: &str,
        bucket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Bucket> {
        self.patch(
            &format!("api/platform/orgs/{}/buckets/{}", org_id, bucket_id),
            body,
        )
        .await
    }

    /// Delete a bucket.
    pub async fn delete_bucket(&self, org_id: &str, bucket_id: &str) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/buckets/{}",
            org_id, bucket_id
        ))
        .await
    }
}
