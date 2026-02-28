use eventsource_stream::Eventsource;
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use reqwest::Method;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::RecordEvent;

impl CopepodClient {
    /// Subscribe to real-time record events for an application.
    ///
    /// Returns a stream of `RecordEvent` items.
    pub async fn subscribe(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<impl Stream<Item = Result<RecordEvent>>> {
        let token = self
            .token_store
            .get()
            .await
            .map(|p| p.token)
            .unwrap_or_default();

        let path = format!(
            "api/realtime/orgs/{}/apps/{}/events",
            org_id, app_id
        );
        let url = self.base_url.join(&path)?;

        let resp = self
            .http
            .request(Method::GET, url)
            .query(&[("access_token", &token)])
            .send()
            .await?
            .error_for_status()
            .map_err(CopepodError::Http)?;

        let stream = resp
            .bytes_stream()
            .map_err(std::io::Error::other)
            .eventsource()
            .map(|result| match result {
                Ok(event) => {
                    if event.event == "record" {
                        serde_json::from_str::<RecordEvent>(&event.data)
                            .map_err(CopepodError::Deserialize)
                    } else {
                        // Skip non-record events (ping, etc.)
                        Err(CopepodError::Sse(format!(
                            "Non-record event: {}",
                            event.event
                        )))
                    }
                }
                Err(e) => Err(CopepodError::Sse(e.to_string())),
            });

        Ok(stream)
    }
}
