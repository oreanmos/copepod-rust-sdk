use eventsource_stream::Eventsource;
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use reqwest::Method;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{RealtimeSubscriptionOptions, RecordEvent};

impl CopepodClient {
    /// Subscribe to real-time record events for an application.
    ///
    /// Returns a stream of `RecordEvent` items.
    pub async fn subscribe(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<impl Stream<Item = Result<RecordEvent>> + Unpin> {
        self.subscribe_with_options(org_id, app_id, RealtimeSubscriptionOptions::default())
            .await
    }

    /// Subscribe to real-time record events with collection, action, and replay
    /// filters.
    ///
    /// App-user events are still constrained by the server-side collection
    /// list rules and field permissions. Access tokens are sent only in the
    /// `Authorization` header and never placed in the request URL.
    pub async fn subscribe_with_options(
        &self,
        org_id: &str,
        app_id: &str,
        options: RealtimeSubscriptionOptions,
    ) -> Result<impl Stream<Item = Result<RecordEvent>> + Unpin> {
        self.subscribe_scoped(org_id, app_id, None, options).await
    }

    /// Shared by the app-level and the Space subscription.
    pub(crate) async fn subscribe_scoped(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: Option<&str>,
        options: RealtimeSubscriptionOptions,
    ) -> Result<impl Stream<Item = Result<RecordEvent>> + Unpin> {
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/realtime");
        let mut request = self.auth_request(Method::GET, &path).await?;
        if let Some(space_id) = space_id {
            request = request.query(&[("space_id", space_id)]);
        }

        let collections = options
            .collections
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",");
        let actions = options
            .actions
            .iter()
            .map(|action| action.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if !collections.is_empty() {
            request = request.query(&[("collections", collections)]);
        }
        if !actions.is_empty() {
            request = request.query(&[("actions", actions)]);
        }
        if let Some(last_event_id) = options.last_event_id {
            request = request.header("Last-Event-ID", last_event_id.to_string());
        }

        let resp = request.send().await?;
        // A Space stream keeps the server's error code (`space_not_found`);
        // the app-level stream keeps its existing transport error.
        let resp = if space_id.is_some() && !resp.status().is_success() {
            return Err(crate::api::space_records::api_error(resp).await);
        } else {
            resp.error_for_status().map_err(CopepodError::Http)?
        };

        let stream = resp
            .bytes_stream()
            .map_err(std::io::Error::other)
            .eventsource()
            .filter_map(|result| async move {
                match result {
                    Ok(event) => {
                        if event.event == "record" {
                            Some(
                                serde_json::from_str::<RecordEvent>(&event.data)
                                    .map_err(CopepodError::Deserialize),
                            )
                        } else {
                            // Ready frames and future control events are not
                            // application records and should stay transparent.
                            None
                        }
                    }
                    Err(error) => Some(Err(CopepodError::Sse(error.to_string()))),
                }
            });

        // Keep the public stream movable even when combinators such as
        // `filter_map` contain an async future that is not `Unpin`. Existing
        // consumers can continue using `stream.next().await` without having
        // to add their own pinning layer.
        Ok(Box::pin(stream))
    }
}
