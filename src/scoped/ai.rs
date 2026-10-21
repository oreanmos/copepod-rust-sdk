use futures_util::stream::Stream;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AiModelsResponse, AiResponse, AiTranscriptionRequest, DecideOutcome, DecideRequest,
    DecideResponse, Donation, DonationConsent, DonationConsentResult, DonationCreated,
    DonationStatus, DonationsDeleted,
};

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

    /// See [`CopepodClient::get_ai_models`].
    pub async fn models(&self) -> Result<AiModelsResponse> {
        self.client
            .get_ai_models(&self.org_id, &self.app_id, &self.collection)
            .await
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

    /// See [`CopepodClient::ai_decide`].
    pub async fn decide(&self, request: &DecideRequest) -> Result<DecideResponse> {
        self.client
            .ai_decide(&self.org_id, &self.app_id, &self.collection, request)
            .await
    }

    /// See [`CopepodClient::ai_decide_with_api_key`].
    pub async fn decide_with_api_key(
        &self,
        request: &DecideRequest,
        user_id: Option<&str>,
    ) -> Result<DecideResponse> {
        self.client
            .ai_decide_with_api_key(&self.org_id, &self.app_id, request, user_id)
            .await
    }

    /// See [`CopepodClient::ai_decide_outcome`].
    pub async fn decide_outcome(&self, decision_id: &str, outcome: &DecideOutcome) -> Result<()> {
        self.client
            .ai_decide_outcome(
                &self.org_id,
                &self.app_id,
                &self.collection,
                decision_id,
                outcome,
            )
            .await
    }

    /// See [`CopepodClient::ai_decide_outcome_with_api_key`].
    pub async fn decide_outcome_with_api_key(
        &self,
        decision_id: &str,
        outcome: &DecideOutcome,
    ) -> Result<()> {
        self.client
            .ai_decide_outcome_with_api_key(&self.org_id, &self.app_id, decision_id, outcome)
            .await
    }

    /// See [`CopepodClient::ai_donations_status`].
    pub async fn donations_status(&self) -> Result<DonationStatus> {
        self.client
            .ai_donations_status(&self.org_id, &self.app_id, &self.collection)
            .await
    }

    /// See [`CopepodClient::ai_set_donation_consent`].
    pub async fn set_donation_consent(
        &self,
        consent: &DonationConsent,
    ) -> Result<DonationConsentResult> {
        self.client
            .ai_set_donation_consent(&self.org_id, &self.app_id, &self.collection, consent)
            .await
    }

    /// See [`CopepodClient::ai_donate`].
    pub async fn donate(&self, donation: &Donation) -> Result<DonationCreated> {
        self.client
            .ai_donate(&self.org_id, &self.app_id, &self.collection, donation)
            .await
    }

    /// See [`CopepodClient::ai_delete_donations`].
    pub async fn delete_donations(&self) -> Result<DonationsDeleted> {
        self.client
            .ai_delete_donations(&self.org_id, &self.app_id, &self.collection)
            .await
    }
}
