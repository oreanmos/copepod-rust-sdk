use futures_util::stream::Stream;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AiResponse, AiTranscriptionRequest};

/// Hosted AI gateway helpers bound to an application and an auth collection.
#[derive(Debug, Clone)]
pub struct ScopedAiClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
    collection: String,
}

impl<'a> ScopedAiClient<'a> {
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

    /// See [`CopepodClient::ai_gateway_base_url`].
    pub fn gateway_base_url(&self) -> String {
        self.client
            .ai_gateway_base_url(&self.org_id, &self.app_id, &self.collection)
    }

    /// See [`CopepodClient::ai_realtime_url`].
    pub fn realtime_url(&self, model: &str) -> Result<String> {
        self.client
            .ai_realtime_url(&self.org_id, &self.app_id, &self.collection, model)
    }

    /// See [`CopepodClient::ai_chat`].
    pub async fn chat(
        &self,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        self.client
            .ai_chat(
                &self.org_id,
                &self.app_id,
                &self.collection,
                body,
                idempotency_key,
            )
            .await
    }

    /// See [`CopepodClient::ai_chat_stream`].
    pub async fn chat_stream(
        &self,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<impl Stream<Item = Result<Value>> + Unpin> {
        self.client
            .ai_chat_stream(
                &self.org_id,
                &self.app_id,
                &self.collection,
                body,
                idempotency_key,
            )
            .await
    }

    /// See [`CopepodClient::ai_embeddings`].
    pub async fn embeddings(
        &self,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        self.client
            .ai_embeddings(
                &self.org_id,
                &self.app_id,
                &self.collection,
                body,
                idempotency_key,
            )
            .await
    }

    /// See [`CopepodClient::ai_ocr`].
    pub async fn ocr(
        &self,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        self.client
            .ai_ocr(
                &self.org_id,
                &self.app_id,
                &self.collection,
                body,
                idempotency_key,
            )
            .await
    }

    /// See [`CopepodClient::ai_upload_file`].
    pub async fn upload_file(
        &self,
        bytes: Vec<u8>,
        filename: &str,
        idempotency_key: Option<&str>,
    ) -> Result<Value> {
        self.client
            .ai_upload_file(
                &self.org_id,
                &self.app_id,
                &self.collection,
                bytes,
                filename,
                idempotency_key,
            )
            .await
    }

    /// See [`CopepodClient::ai_file_url`].
    pub async fn file_url(&self, file_id: &str, expiry_hours: u32) -> Result<Value> {
        self.client
            .ai_file_url(
                &self.org_id,
                &self.app_id,
                &self.collection,
                file_id,
                expiry_hours,
            )
            .await
    }

    /// See [`CopepodClient::ai_delete_file`].
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        self.client
            .ai_delete_file(&self.org_id, &self.app_id, &self.collection, file_id)
            .await
    }

    /// See [`CopepodClient::ai_transcribe`].
    pub async fn transcribe(
        &self,
        request: AiTranscriptionRequest,
        idempotency_key: Option<&str>,
    ) -> Result<AiResponse<Value>> {
        self.client
            .ai_transcribe(
                &self.org_id,
                &self.app_id,
                &self.collection,
                request,
                idempotency_key,
            )
            .await
    }

    /// See [`CopepodClient::ai_moderations`].
    pub async fn moderations(&self, body: &Value) -> Result<Value> {
        self.client
            .ai_moderations(&self.org_id, &self.app_id, &self.collection, body)
            .await
    }

    /// See [`CopepodClient::ai_chat_moderations`].
    pub async fn chat_moderations(&self, body: &Value) -> Result<Value> {
        self.client
            .ai_chat_moderations(&self.org_id, &self.app_id, &self.collection, body)
            .await
    }
}
