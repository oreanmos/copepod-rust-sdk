//! Analytics ingestion (launch-g2): `track_events` posts app events to
//! `api/platform/orgs/{org}/apps/{app}/analytics/events` with the client's API
//! key. See `docs/analytics.md`.

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AnalyticsEvent, TrackEventsResponse};

impl CopepodClient {
    /// Send a batch of analytics events via
    /// `POST api/platform/orgs/{org}/apps/{app}/analytics/events`.
    ///
    /// Requires an API key with scope `analytics:write`; no Bearer token is
    /// sent. The batch holds 1–100 events; batch size, event names, props and
    /// ids are validated before any network call and rejected with
    /// [`CopepodError::InvalidArgument`]. The server answers `202` with
    /// per-event acceptances, filtered counts and rejections; non-2xx
    /// responses arrive as [`CopepodError::Api`] with the server's code
    /// preserved (for example `batch_too_large`).
    pub async fn track_events(
        &self,
        org_id: &str,
        app_id: &str,
        events: &[AnalyticsEvent],
    ) -> Result<TrackEventsResponse> {
        // The request lands with the implementation (launch-g2 S8); touch the
        // arguments so the stub compiles without warnings.
        let _ = (self, org_id, app_id, events);
        unimplemented!("launch-g2 S8")
    }
}
