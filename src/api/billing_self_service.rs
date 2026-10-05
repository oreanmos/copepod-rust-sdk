use reqwest::Method;

use crate::error::Result;
use crate::{
    CopepodClient, CopepodError, MeBillingSummary, MeCheckoutRequest, MeCheckoutResponse,
    MePaymentRecoveryRequest, MePaymentRecoveryResponse, MeReceiptList, MeTrialStartRequest,
    MeTrialStartResponse,
};

fn validate_idempotency_key(key: &str) -> Result<()> {
    if key.is_empty()
        || key.len() > 160
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err(CopepodError::InvalidArgument(
            "Idempotency key must be 1–160 ASCII letters, digits, '.', ':', '_' or '-'".into(),
        ));
    }
    Ok(())
}

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
    /// (missing or empty is a 400 with no error code), and it must be an absolute HTTPS URL (400
    /// otherwise; plain `http` only for localhost). Errors: 422
    /// `billing_interval_unavailable`, `billing_country_not_supported`,
    /// `discount_code_invalid`, `discount_code_exhausted`; 409
    /// `subscription_active` (a paid subscription is already active or in
    /// grace, or the user has a live Mollie subscription, including a card
    /// trial); 409 when the `Idempotency-Key` was already used with a
    /// different body; 409 `checkout_pending` while an earlier checkout is open
    /// and its Mollie payment has not expired (a replay with the same key and
    /// body still returns the original response). The `checkout_pending` body
    /// carries `checkout_url` (nullable) and `expires_at` at its top level;
    /// read them with [`CopepodError::checkout_pending_details`].
    /// Invalid arguments fail offline. Older
    /// servers return 404.
    pub async fn me_start_checkout(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        idempotency_key: &str,
        body: &MeCheckoutRequest,
    ) -> Result<MeCheckoutResponse> {
        validate_idempotency_key(idempotency_key)?;
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

    /// Billing summary of the signed-in app user, covering every subscription
    /// state (`none`, `trialing`, `active`, `past_due`, `cancelled`, `expired`).
    ///
    /// `GET .../auth/{collection}/me/billing` with the app-user bearer token.
    /// `next_charge` is null unless a Mollie subscription exists. Another
    /// app's token is a 401. Older servers return 404.
    pub async fn me_billing_summary(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<MeBillingSummary> {
        self.get(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/billing"
        ))
        .await
    }

    /// The signed-in app user's receipts, newest first.
    ///
    /// `GET .../me/billing/receipts` with the app-user bearer token. `limit`
    /// must be 1-100 and `offset` 0 or more (the server answers 400
    /// otherwise); both are validated offline and sent only when `Some`. The
    /// server defaults are limit 50, offset 0. Older servers return 404.
    pub async fn me_list_receipts(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<MeReceiptList> {
        if limit.is_some_and(|l| !(1..=100).contains(&l)) {
            return Err(CopepodError::InvalidArgument(
                "limit must be between 1 and 100".into(),
            ));
        }
        if offset.is_some_and(|o| o < 0) {
            return Err(CopepodError::InvalidArgument(
                "offset must be 0 or more".into(),
            ));
        }
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if let Some(o) = offset {
            params.push(format!("offset={o}"));
        }
        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };
        self.get(&format!(
            "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/billing/receipts{query}"
        ))
        .await
    }

    /// One of the caller's receipts as a standalone HTML page.
    ///
    /// `GET .../me/billing/receipts/{receipt_id}` with the app-user bearer
    /// token. `receipt_id` must be non-empty and contain no `/`. Errors: 404
    /// `receipt_not_found` (also for another user's or app's receipt). Older
    /// servers return 404 without that code.
    pub async fn me_get_receipt_html(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        receipt_id: &str,
    ) -> Result<String> {
        if receipt_id.is_empty() || receipt_id.contains('/') {
            return Err(CopepodError::InvalidArgument(
                "receipt_id must be non-empty and contain no '/'".into(),
            ));
        }
        let resp = self
            .auth_request(
                Method::GET,
                &format!(
                    "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/billing/receipts/{receipt_id}"
                ),
            )
            .await?
            .send()
            .await?;
        if resp.status().is_success() {
            return Ok(resp.text().await?);
        }
        CopepodClient::handle_response_pub::<String>(resp).await
    }

    /// Start a checkout for the outstanding amount of a past-due subscription.
    ///
    /// `POST .../me/billing/payment-recovery` with the app-user bearer token
    /// and an `Idempotency-Key` (1-160 ASCII letters, digits, `.:_-`). While
    /// the episode's earlier recovery payment is open the same checkout is
    /// returned. `redirect_url` must be non-empty (and an absolute HTTPS URL
    /// server-side). Errors: 409 `not_past_due`, `recovery_unavailable`,
    /// `recovery_processing` (a recovery payment is being processed; poll
    /// [`CopepodClient::me_billing_summary`]); 422 `redirect_url_not_allowed`.
    /// While past due, `me_start_checkout` answers 409 `subscription_past_due`.
    /// Older servers return 404.
    pub async fn me_start_payment_recovery(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        idempotency_key: &str,
        body: &MePaymentRecoveryRequest,
    ) -> Result<MePaymentRecoveryResponse> {
        validate_idempotency_key(idempotency_key)?;
        if body.redirect_url.trim().is_empty() {
            return Err(CopepodError::InvalidArgument(
                "redirect_url must not be empty".into(),
            ));
        }
        let response = self
            .auth_request(
                Method::POST,
                &format!(
                    "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/billing/payment-recovery"
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
