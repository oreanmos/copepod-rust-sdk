//! Analytics helpers bound to an organization and application (launch-g2).

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AnalyticsEvent, TrackEventsResponse};

/// Analytics helpers bound to an organization and application. Calls
/// authenticate with the client's API key.
#[derive(Debug, Clone)]
pub struct ScopedAnalyticsClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
}

impl<'a> ScopedAnalyticsClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: &str, app_id: &str) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// Send a batch of analytics events; see
    /// [`CopepodClient::track_events`](crate::CopepodClient::track_events).
    pub async fn track(&self, events: &[AnalyticsEvent]) -> Result<TrackEventsResponse> {
        // Delegation lands with the implementation (launch-g2 S8); touch the
        // bound fields so the stub compiles without lint suppressions.
        let _ = (self.client, &self.org_id, &self.app_id, events);
        unimplemented!("launch-g2 S8")
    }
}
