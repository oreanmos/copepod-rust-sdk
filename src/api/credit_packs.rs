//! One-off credit packs of the signed-in app user (`me/credits/packs*`,
//! `me/credits/purchases*`). App-user bearer token. Credits are added by the
//! server's payment webhook, so poll the purchase until it is `paid`.

use reqwest::Method;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    CreditPackCatalog, CreditPackPurchase, CreditPackPurchaseList, CreditPackPurchaseRequest,
};

fn base(org_id: &str, app_id: &str, collection: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/credits")
}

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

fn validate_segment(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() || value.contains(['/', '?', '#']) {
        return Err(CopepodError::InvalidArgument(format!(
            "{name} must be a non-empty id"
        )));
    }
    Ok(())
}

impl CopepodClient {
    /// The credit packs on sale, cheapest first.
    pub async fn list_packs(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<CreditPackCatalog> {
        self.get(&format!("{}/packs", base(org_id, app_id, collection)))
            .await
    }

    /// Start buying a pack; returns a `pending` purchase with the checkout URL.
    ///
    /// `POST .../me/credits/packs/{pack_id}/purchase` with an `Idempotency-Key`
    /// (1-160 ASCII letters, digits, `.:_-`): a retry with the same key returns
    /// the same purchase and link. Errors: 404 `pack_unavailable`, 409
    /// `idempotency_key_reused` (same key, another pack), 422
    /// `redirect_url_not_allowed`, 503 while the server upgrades. Invalid
    /// arguments fail before any request.
    pub async fn purchase_credit_pack(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        pack_id: &str,
        idempotency_key: &str,
        body: &CreditPackPurchaseRequest,
    ) -> Result<CreditPackPurchase> {
        validate_segment("pack_id", pack_id)?;
        validate_idempotency_key(idempotency_key)?;
        let body = CreditPackPurchaseRequest {
            redirect_url: body.redirect_url.trim().to_string(),
        };
        if body.redirect_url.is_empty() {
            return Err(CopepodError::InvalidArgument(
                "redirect_url must not be empty".into(),
            ));
        }
        let response = self
            .auth_request(
                Method::POST,
                &format!(
                    "{}/packs/{pack_id}/purchase",
                    base(org_id, app_id, collection)
                ),
            )
            .await?
            .header("Idempotency-Key", idempotency_key)
            .json(&body)
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }

    /// The caller's pack purchases, newest first. The server defaults `limit`
    /// to 50 and caps it at 100.
    pub async fn list_credit_purchases(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<CreditPackPurchaseList> {
        let mut params: Vec<(&str, u32)> = Vec::new();
        if let Some(limit) = limit {
            params.push(("limit", limit));
        }
        if let Some(offset) = offset {
            params.push(("offset", offset));
        }
        let response = self
            .auth_request(
                Method::GET,
                &format!("{}/purchases", base(org_id, app_id, collection)),
            )
            .await?
            .query(&params)
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }

    /// One of the caller's pack purchases; 404 for another user's or unknown id.
    pub async fn get_credit_purchase(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        purchase_id: &str,
    ) -> Result<CreditPackPurchase> {
        validate_segment("purchase_id", purchase_id)?;
        self.get(&format!(
            "{}/purchases/{purchase_id}",
            base(org_id, app_id, collection)
        ))
        .await
    }
}
