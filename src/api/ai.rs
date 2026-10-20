//! Hosted AI gateway: Mistral-shaped pass-through routes metered in user credits.

use eventsource_stream::Eventsource;
use futures_util::stream::{self, Stream, StreamExt, TryStreamExt};
use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::api::spaces::validate_id;
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{AiModelList, AiModelsResponse, AiResponse, AiTranscriptionRequest};

fn invalid<T>(message: &str) -> Result<T> {
    Err(CopepodError::InvalidArgument(message.into()))
}

fn validate_body(body: &Value) -> Result<()> {
    let Some(object) = body.as_object() else {
        return invalid("AI request body must be a JSON object");
    };
    match object.get("model") {
        None | Some(Value::Null) => {}
        Some(Value::String(model)) if !model.is_empty() => {}
        Some(Value::String(_)) => return invalid("AI request `model` must not be empty"),
        Some(_) => return invalid("AI request `model` must be a string"),
    }
    Ok(())
}

/// Reject a body that asks for streaming on a method that returns one JSON answer.
fn reject_stream_flag(body: &Value) -> Result<()> {
    if body.get("stream").and_then(Value::as_bool) == Some(true) {
        return invalid("`stream: true` needs ai_chat_stream");
    }
    Ok(())
}

fn validate_idempotency_key(key: Option<&str>) -> Result<()> {
    if let Some(key) = key {
        let ok = (1..=160).contains(&key.len())
            && key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'-'));
        if !ok {
            return invalid(
                "Idempotency-Key must be 1 to 160 characters of letters, digits, '_', '.', ':' or '-'",
            );
        }
    }
    Ok(())
}

fn validate_file_id(file_id: &str) -> Result<()> {
    let ok = (1..=128).contains(&file_id.len())
        && file_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'));
    if !ok {
        return invalid("file_id must be 1 to 128 characters of letters, digits, '_' or '-'");
    }
    Ok(())
}

fn user_prefix(org_id: &str, app_id: &str, collection: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/ai/v1")
}

fn header_string(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// Read a gateway answer: map errors, parse the body and the metering headers.
async fn ai_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<AiResponse<T>> {
    if !resp.status().is_success() {
        return Err(CopepodClient::error_from_response(resp).await);
    }
    let request_id = header_string(&resp, "x-copepod-request-id");
    let charged_credits =
        header_string(&resp, "x-copepod-credits-charged").and_then(|v| v.parse().ok());
    let remaining_credits =
        header_string(&resp, "x-copepod-credits-remaining").and_then(|v| v.parse().ok());
    let estimated = header_string(&resp, "x-copepod-usage-estimated").as_deref() == Some("true");
    let body = resp.json::<T>().await?;
    Ok(AiResponse {
        body,
        request_id,
        charged_credits,
        remaining_credits,
        estimated,
    })
}

/// Turn an in-stream `{"error": {...}}` event into an API error.
fn stream_error(value: &Value) -> CopepodError {
    let error = &value["error"];
    let code = error.get("code").and_then(Value::as_str).map(String::from);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| error.as_str())
        .unwrap_or("AI stream failed")
        .to_string();
    let status = match code.as_deref() {
        Some("ai_provider_unavailable") => 503,
        Some("ai_provider_rate_limited" | "ai_rate_limited") => 429,
        _ => 502,
    };
    CopepodError::Api {
        status,
        code,
        message,
    }
}

impl CopepodClient {
    /// Base URL for a Mistral-compatible client: `{copepod}/api/platform/orgs/{org}/apps/{app}/auth/{collection}/me/ai`.
    /// Set the client's bearer to the app-user access token.
    pub fn ai_gateway_base_url(&self, org_id: &str, app_id: &str, collection: &str) -> String {
        let prefix = user_prefix(org_id, app_id, collection);
        let prefix = prefix.trim_end_matches("/v1");
        format!("{}{prefix}", self.base_url.as_str())
    }

    /// `GET …/me/ai/models` (bearer): the hosted models the caller's plan offers.
    ///
    /// A 404 without a JSON `message`/`code` body (a node older than the route) is not an error: it returns
    /// [`AiModelsResponse::Unavailable`] so callers keep their last list. A plan
    /// without hosted AI is `Available` with `hosted_ai_enabled: false`.
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id; 401 and other statuses as `CopepodError::Api`.
    pub async fn get_ai_models(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<AiModelsResponse> {
        validate_id("org_id", org_id)?;
        validate_id("app_id", app_id)?;
        validate_id("collection", collection)?;
        let path =
            format!("api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/ai/models");
        let resp = self.auth_request(Method::GET, &path).await?.send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            // An old node has no route: axum's empty (or non-JSON) 404. A Copepod
            // JSON error body (unknown app, etc.) is a real error.
            let bytes = resp.bytes().await.unwrap_or_default();
            let is_api_error = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| {
                    v.as_object()
                        .map(|o| o.contains_key("message") || o.contains_key("code"))
                })
                .unwrap_or(false);
            if !is_api_error {
                return Ok(AiModelsResponse::Unavailable);
            }
            let (code, message) = crate::client::decode_error_body(&bytes);
            return Err(CopepodError::Api {
                status: 404,
                code,
                message,
            });
        }
        Self::handle_response_pub::<AiModelList>(resp)
            .await
            .map(AiModelsResponse::Available)
    }

    /// WebSocket URL of the realtime transcription route (`ws`/`wss` scheme).
    /// The access token is never placed in the URL: send it as a bearer header on the upgrade.
    /// The realtime route answers 404 until the Copepod C3 slice is deployed.
    ///
    /// # Errors
    /// `InvalidArgument` when `model` is empty.
    pub fn ai_realtime_url(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        model: &str,
    ) -> Result<String> {
        if model.trim().is_empty() {
            return invalid("model must not be empty");
        }
        let mut url = self.base_url.join(&format!(
            "{}/audio/transcriptions/realtime",
            user_prefix(org_id, app_id, collection)
        ))?;
        let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
        url.set_scheme(scheme).map_err(|_| {
            CopepodError::InvalidArgument("base URL cannot use a WebSocket scheme".into())
        })?;
        url.query_pairs_mut().append_pair("model", model);
        Ok(url.into())
    }

    async fn ai_post_json(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        route: &str,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<reqwest::Response> {
        validate_body(body)?;
        validate_idempotency_key(idempotency_key)?;
        let path = format!("{}{route}", user_prefix(org_id, app_id, collection));
        let mut request = self.auth_request(Method::POST, &path).await?.json(body);
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        Ok(request.send().await?)
    }

    /// `POST …/me/ai/v1/chat/completions` (bearer). Sends `Idempotency-Key` when given.
    ///
    /// Errors: 402 `ai_credits_exhausted`, 403 `hosted_ai_disabled` / `ai_model_not_allowed`,
    /// 409 `ai_request_replayed` (the key was already charged, or is still running),
    /// 429 `ai_rate_limited`, 503 `ai_provider_unavailable` (the shared rate limiter being unavailable is a 503 without a code).
    pub async fn ai_chat(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        validate_body(body)?;
        reject_stream_flag(body)?;
        let resp = self
            .ai_post_json(
                org_id,
                app_id,
                collection,
                "/chat/completions",
                body,
                idempotency_key,
            )
            .await?;
        ai_response(resp).await
    }

    /// Streaming chat: forces `stream: true`, yields each SSE JSON chunk and stops at `[DONE]`.
    /// An `{"error":…}` event yields one `CopepodError::Api` and ends the stream. A stream that
    /// closes before `[DONE]` yields one `CopepodError::Sse("stream ended before [DONE]")`; a
    /// `Deserialize` error mid-stream also means the answer was interrupted (the server can garble
    /// an error event when the provider fails mid-event), so treat both as an interrupted answer. Only the request
    /// id header is available; credits are debited after the last byte (read them with `get_current_credits`).
    pub async fn ai_chat_stream(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<impl Stream<Item = Result<Value>> + Unpin> {
        validate_body(body)?;
        let mut body = body.clone();
        body["stream"] = Value::Bool(true);
        let resp = self
            .ai_post_json(
                org_id,
                app_id,
                collection,
                "/chat/completions",
                &body,
                idempotency_key,
            )
            .await?;
        if !resp.status().is_success() {
            return Err(CopepodClient::error_from_response(resp).await);
        }
        let events = resp
            .bytes_stream()
            .map_err(std::io::Error::other)
            .eventsource();
        let stream = stream::unfold((Box::pin(events), false), |(mut events, done)| async move {
            if done {
                return None;
            }
            loop {
                let Some(next) = events.next().await else {
                    // Closed without `[DONE]`: the answer was cut off.
                    return Some((
                        Err(CopepodError::Sse("stream ended before [DONE]".into())),
                        (events, true),
                    ));
                };
                match next {
                    Err(error) => {
                        return Some((Err(CopepodError::Sse(error.to_string())), (events, true)))
                    }
                    Ok(event) => {
                        let data = event.data.trim();
                        if data == "[DONE]" {
                            return None;
                        }
                        if data.is_empty() {
                            continue;
                        }
                        return Some(match serde_json::from_str::<Value>(data) {
                            Ok(value) if value.get("error").is_some() => {
                                (Err(stream_error(&value)), (events, true))
                            }
                            Ok(value) => (Ok(value), (events, false)),
                            Err(error) => (Err(CopepodError::Deserialize(error)), (events, true)),
                        });
                    }
                }
            }
        });
        Ok(Box::pin(stream))
    }

    /// `POST …/me/ai/v1/embeddings` (bearer).
    pub async fn ai_embeddings(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        let resp = self
            .ai_post_json(
                org_id,
                app_id,
                collection,
                "/embeddings",
                body,
                idempotency_key,
            )
            .await?;
        ai_response(resp).await
    }

    /// `POST …/me/ai/v1/ocr` (bearer); metered in pages.
    pub async fn ai_ocr(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        let resp = self
            .ai_post_json(org_id, app_id, collection, "/ocr", body, idempotency_key)
            .await?;
        ai_response(resp).await
    }

    /// `POST …/me/ai/v1/files` (bearer, multipart `purpose=ocr` + `file`). Not metered.
    pub async fn ai_upload_file(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        bytes: Vec<u8>,
        filename: &str,
        idempotency_key: Option<&str>,
    ) -> Result<Value> {
        validate_idempotency_key(idempotency_key)?;
        if filename.trim().is_empty() {
            return invalid("filename must not be empty");
        }
        let path = format!("{}/files", user_prefix(org_id, app_id, collection));
        let form = Form::new()
            .text("purpose", "ocr")
            .part("file", Part::bytes(bytes).file_name(filename.to_string()));
        let mut request = self
            .auth_request(Method::POST, &path)
            .await?
            .multipart(form);
        if let Some(key) = idempotency_key {
            request = request.header("Idempotency-Key", key);
        }
        Self::handle_response_pub(request.send().await?).await
    }

    /// `GET …/me/ai/v1/files/{file_id}/url?expiry=1..24` (bearer); owner only.
    pub async fn ai_file_url(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        file_id: &str,
        expiry_hours: u32,
    ) -> Result<Value> {
        validate_file_id(file_id)?;
        if !(1..=24).contains(&expiry_hours) {
            return invalid("expiry_hours must be between 1 and 24");
        }
        let path = format!(
            "{}/files/{file_id}/url",
            user_prefix(org_id, app_id, collection)
        );
        let resp = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&[("expiry", expiry_hours)])
            .send()
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `DELETE …/me/ai/v1/files/{file_id}` (bearer); owner only.
    pub async fn ai_delete_file(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        file_id: &str,
    ) -> Result<()> {
        validate_file_id(file_id)?;
        let path = format!(
            "{}/files/{file_id}",
            user_prefix(org_id, app_id, collection)
        );
        let resp = self
            .auth_request(Method::DELETE, &path)
            .await?
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(CopepodClient::error_from_response(resp).await)
        }
    }

    /// `POST …/me/ai/v1/audio/transcriptions` (bearer, multipart); metered in audio seconds.
    pub async fn ai_transcribe(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        request: AiTranscriptionRequest,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        validate_idempotency_key(idempotency_key)?;
        if request.model.trim().is_empty() {
            return invalid("model must not be empty");
        }
        if request.filename.trim().is_empty() {
            return invalid("filename must not be empty");
        }
        if request
            .fields
            .iter()
            .any(|(name, _)| name == "model" || name == "file")
        {
            return invalid("fields must not contain `model` or `file`");
        }
        if request
            .fields
            .iter()
            .any(|(name, value)| name == "stream" && value.eq_ignore_ascii_case("true"))
        {
            return invalid("streaming transcription is not supported by ai_transcribe");
        }
        let path = format!(
            "{}/audio/transcriptions",
            user_prefix(org_id, app_id, collection)
        );
        let mut form = Form::new().text("model", request.model);
        for (name, value) in request.fields {
            form = form.text(name, value);
        }
        form = form.part(
            "file",
            Part::bytes(request.bytes).file_name(request.filename),
        );
        let mut http = self
            .auth_request(Method::POST, &path)
            .await?
            .multipart(form);
        if let Some(key) = idempotency_key {
            http = http.header("Idempotency-Key", key);
        }
        ai_response(http.send().await?).await
    }

    /// `POST …/me/ai/v1/moderations` (bearer). Moderation models only (`mistral-moderation*`);
    /// charged to the app as overhead, not to the user.
    pub async fn ai_moderations(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &Value,
    ) -> Result<Value> {
        let resp = self
            .ai_post_json(org_id, app_id, collection, "/moderations", body, None)
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `POST …/me/ai/v1/chat/moderations` (bearer); the chat-message form of [`Self::ai_moderations`].
    pub async fn ai_chat_moderations(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &Value,
    ) -> Result<Value> {
        let resp = self
            .ai_post_json(org_id, app_id, collection, "/chat/moderations", body, None)
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `POST /api/platform/orgs/{org}/apps/{app}/ai/v1/{moderations|chat/moderations}`.
    ///
    /// Sends `X-API-Key` only, never `Authorization` (scope `ai:moderate`). `user_id` is sent as
    /// `X-User-Id` for attribution only. `chat` selects the chat-message route.
    pub async fn app_ai_moderations(
        &self,
        org_id: &str,
        app_id: &str,
        body: &Value,
        chat: bool,
        user_id: Option<&str>,
    ) -> Result<Value> {
        validate_body(body)?;
        let route = if chat {
            "chat/moderations"
        } else {
            "moderations"
        };
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/ai/v1/{route}");
        let mut request = self.api_key_request(Method::POST, &path)?.json(body);
        if let Some(user_id) = user_id {
            request = request.header("X-User-Id", user_id);
        }
        Self::handle_response_pub(request.send().await?).await
    }
}
