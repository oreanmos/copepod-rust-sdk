use reqwest::Method;

use crate::error::Result;
use crate::{
    CopepodClient, CopepodError, MeCheckoutRequest, MeCheckoutResponse, MeTrialStartRequest,
    MeTrialStartResponse,
};

impl CopepodClient {
    /// Start the signed-in app user's no-card trial.
    ///
    /// `POST api/platform/orgs/{org}/apps/{app}/auth/{collection}/me/subscription/trial`
    /// with the app-user bearer token. Errors: `billing_country_not_supported`
    /// (422), `subscription_exists` (409), `trial_already_used` (409),
    /// `trial_unavailable` (422; also returned when the app is in card-trial
    /// mode and offers no no-card trial). Older servers return 404.
    pub async fn me_start_trial(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &MeTrialStartRequest,
    ) -> Result<MeTrialStartResponse> {
        self.post(
            &format!(
                "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/subscription/trial"
            ),
            body,
        )
        .await
    }

    /// Start a paid checkout for the signed-in app user.
    ///
    /// `POST .../auth/{collection}/me/subscription/checkout` with the app-user
    /// bearer token and an `Idempotency-Key` (1-160 ASCII letters, digits,
    /// `.:_-`); reuse the key and unchanged body to recover a lost response.
    /// A present `redirect_url` must be non-empty; current servers require it
    /// (missing or empty is 400), and it must be an absolute HTTPS URL (400
    /// otherwise; plain `http` only for localhost). Errors: 422
    /// `billing_interval_unavailable`, `billing_country_not_supported`,
    /// `discount_code_invalid`, `discount_code_exhausted`; 409
    /// `subscription_active` (a paid subscription is already active or in
    /// grace); 409 when the `Idempotency-Key` was already used with a
    /// different body; 409 when an open checkout already exists (see the
    /// server docs for its code). Invalid arguments fail offline. Older
    /// servers return 404.
    pub async fn me_start_checkout(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        idempotency_key: &str,
        body: &MeCheckoutRequest,
    ) -> Result<MeCheckoutResponse> {
        if idempotency_key.is_empty()
            || idempotency_key.len() > 160
            || !idempotency_key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
        {
            return Err(CopepodError::InvalidArgument(
                "Idempotency key must be 1–160 ASCII letters, digits, '.', ':', '_' or '-'".into(),
            ));
        }
        if body
            .redirect_url
            .as_deref()
            .is_some_and(|u| u.trim().is_empty())
        {
            return Err(CopepodError::InvalidArgument(
                "redirect_url must not be empty".into(),
            ));
        }
        let response = self
            .auth_request(
                Method::POST,
                &format!(
                    "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/subscription/checkout"
                ),
            )
            .await?
            .header("Idempotency-Key", idempotency_key)
            .json(body)
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }
}
