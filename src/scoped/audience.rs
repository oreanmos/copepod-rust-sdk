use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ActiveAnnouncement, CreateLeadRequest, CreateLeadResponse};

/// Audience helpers (leads and announcements) bound to an organization and
/// application. Both calls authenticate with the client's API key.
#[derive(Debug, Clone)]
pub struct ScopedAudienceClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedAudienceClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: &str, app_id: &str) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// Add a visitor to an audience list; see [`CopepodClient::create_lead`].
    pub async fn create_lead(&self, lead: &CreateLeadRequest) -> Result<CreateLeadResponse> {
        self.client
            .create_lead(&self.org_id, &self.app_id, lead)
            .await
    }

    /// List active announcements; see [`CopepodClient::list_active_announcements`].
    pub async fn active_announcements(&self) -> Result<Vec<ActiveAnnouncement>> {
        self.client
            .list_active_announcements(&self.org_id, &self.app_id)
            .await
    }
}
