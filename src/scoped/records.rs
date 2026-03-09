use serde::Serialize;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::query::RecordQueryBuilder;

/// Record helpers bound to a specific collection.
#[derive(Debug, Clone)]
pub struct ScopedRecordCollectionClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
    collection: String,
}

impl<'a> ScopedRecordCollectionClient<'a> {
    pub(crate) fn new(
        client: &'a CopepodClient,
        org_id: &str,
        app_id: &str,
        collection: impl Into<String>,
    ) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
            collection: collection.into(),
        }
    }

    /// Return the bound collection name.
    pub fn collection(&self) -> &str {
        &self.collection
    }

    /// Start building a query for this collection.
    pub fn query(&self) -> RecordQueryBuilder<'a> {
        self.client
            .records(&self.org_id, &self.app_id, &self.collection)
    }

    /// Create a new record in this collection.
    pub async fn create(&self, body: &impl Serialize) -> Result<Value> {
        self.client
            .create_record(&self.org_id, &self.app_id, &self.collection, body)
            .await
    }

    /// Update an existing record in this collection.
    pub async fn update(&self, record_id: &str, body: &impl Serialize) -> Result<Value> {
        self.client
            .update_record(
                &self.org_id,
                &self.app_id,
                &self.collection,
                record_id,
                body,
            )
            .await
    }

    /// Delete a record from this collection.
    pub async fn delete(&self, record_id: &str) -> Result<()> {
        self.client
            .delete_record(&self.org_id, &self.app_id, &self.collection, record_id)
            .await
    }
}
