use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{Collection, ListResult};

impl CopepodClient {
    /// List all collections in an app.
    pub async fn list_collections(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ListResult<Collection>> {
        self.get(&format!(
            "api/orgs/{}/apps/{}/collections",
            org_id, app_id
        ))
        .await
    }

    /// Get a collection by ID or name.
    pub async fn get_collection(
        &self,
        org_id: &str,
        app_id: &str,
        collection_id: &str,
    ) -> Result<Collection> {
        self.get(&format!(
            "api/orgs/{}/apps/{}/collections/{}",
            org_id, app_id, collection_id
        ))
        .await
    }

    /// Create a new collection.
    pub async fn create_collection(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Collection> {
        self.post(
            &format!("api/orgs/{}/apps/{}/collections", org_id, app_id),
            body,
        )
        .await
    }

    /// Update a collection.
    pub async fn update_collection(
        &self,
        org_id: &str,
        app_id: &str,
        collection_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Collection> {
        self.patch(
            &format!(
                "api/orgs/{}/apps/{}/collections/{}",
                org_id, app_id, collection_id
            ),
            body,
        )
        .await
    }

    /// Delete a collection.
    pub async fn delete_collection(
        &self,
        org_id: &str,
        app_id: &str,
        collection_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/orgs/{}/apps/{}/collections/{}",
            org_id, app_id, collection_id
        ))
        .await
    }
}
