use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::query::RecordQueryBuilder;

impl CopepodClient {
    /// Start building a record query for a collection.
    pub fn records<'a>(
        &'a self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> RecordQueryBuilder<'a> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/records/{}",
            org_id, app_id, collection
        );
        RecordQueryBuilder::new(self, path)
    }

    /// Create a new record in a collection.
    pub async fn create_record(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/records/{}",
            org_id, app_id, collection
        );
        self.post(&path, body).await
    }

    /// Update an existing record.
    pub async fn update_record(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        record_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/records/{}/{}",
            org_id, app_id, collection, record_id
        );
        self.patch(&path, body).await
    }

    /// Delete a record.
    pub async fn delete_record(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        record_id: &str,
    ) -> Result<()> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/records/{}/{}",
            org_id, app_id, collection, record_id
        );
        self.delete(&path).await
    }
}
