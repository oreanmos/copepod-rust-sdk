use crate::client::CopepodClient;

use super::{ScopedAppAuthClient, ScopedMigrationClient, ScopedRecordCollectionClient};

/// Application-scoped client helpers.
#[derive(Debug, Clone)]
pub struct ScopedAppClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedAppClient<'a> {
    pub(crate) fn new(
        client: &'a CopepodClient,
        org_id: impl Into<String>,
        app_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            org_id: org_id.into(),
            app_id: app_id.into(),
        }
    }

    /// Return the bound organization ID.
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    /// Return the bound application ID.
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Return auth helpers bound to a specific app auth collection.
    pub fn auth(&self, collection: impl Into<String>) -> ScopedAppAuthClient<'a> {
        ScopedAppAuthClient::new(self.client, &self.org_id, &self.app_id, collection)
    }

    /// Return record helpers bound to a specific collection.
    pub fn records(&self, collection: impl Into<String>) -> ScopedRecordCollectionClient<'a> {
        ScopedRecordCollectionClient::new(self.client, &self.org_id, &self.app_id, collection)
    }

    /// Return migration helpers bound to this application.
    pub fn migrations(&self) -> ScopedMigrationClient<'a> {
        ScopedMigrationClient::new(self.client, &self.org_id, &self.app_id)
    }
}
