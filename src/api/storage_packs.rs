//! Monthly storage packs of the signed-in app user (`me/storage-packs*`).
//! App-user bearer token. Each subscription is its own provider subscription
//! charged on the plan's payment mandate; the space is granted only once the
//! server's webhook confirms the first payment, so re-read the catalog until
//! the pack is `active`. See `docs/storage-packs.md`.

use reqwest::Method;

use super::credit_packs::{validate_idempotency_key, validate_segment};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{StoragePackCatalog, StoragePackSubscribeRequest, StoragePackSubscription};

/// Largest quantity the server accepts in one subscribe.
pub const STORAGE_PACK_MAX_QUANTITY: i64 = 100;

fn base(org_id: &str, app_id: &str, collection: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/storage-packs")
}

impl CopepodClient {
    /// The storage packs on sale, the caller's own packs and whether the app
    /// has capacity left (`GET .../me/storage-packs`).
    pub async fn list_storage_packs(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<StoragePackCatalog> {
        self.get(&base(org_id, app_id, collection)).await
    }

    /// Subscribe to `quantity` units of a pack
    /// (`POST .../me/storage-packs/{pack_id}/subscribe`).
    ///
    /// Sends an `Idempotency-Key` (1-160 ASCII letters, digits, `.:_-`): a
    /// retry with the same key returns the same subscription. The result is
    /// `pending` until the first payment is confirmed. Errors (see the
    /// `*_CODE` constants): 402 `requires_paid_plan`, 404 `pack_unavailable`,
    /// 409 `currency_mismatch`, `storage_unlimited`, `capacity_unavailable`,
    /// `mandate_unavailable` (retry with the same key later),
    /// `idempotency_key_reused`, 502 `payment_provider_rejected`, 503
    /// `packs_upgrading`. A `quantity` outside 1..=100 or a bad key or id
    /// fails before any request.
    pub async fn subscribe_storage_pack(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        pack_id: &str,
        quantity: i64,
        idempotency_key: &str,
    ) -> Result<StoragePackSubscription> {
        validate_segment("pack_id", pack_id)?;
        validate_idempotency_key(idempotency_key)?;
        if !(1..=STORAGE_PACK_MAX_QUANTITY).contains(&quantity) {
            return Err(CopepodError::InvalidArgument(format!(
                "quantity must be between 1 and {STORAGE_PACK_MAX_QUANTITY}"
            )));
        }
        let response = self
            .auth_request(
                Method::POST,
                &format!("{}/{pack_id}/subscribe", base(org_id, app_id, collection)),
            )
            .await?
            .header("Idempotency-Key", idempotency_key)
            .json(&StoragePackSubscribeRequest { quantity })
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }

    /// Stop a pack's monthly charge
    /// (`DELETE .../me/storage-packs/{subscription_id}`, the id from
    /// `owned_storage[].subscription_id` or `pack_subscription_id`).
    ///
    /// A paid pack becomes `cancelling` and keeps its space until `ends_at`; a
    /// pack never paid becomes `cancelled`. A repeat answers the same. Errors:
    /// 404 for an unknown or another user's pack, 503 `packs_upgrading`.
    pub async fn cancel_storage_pack(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        subscription_id: &str,
    ) -> Result<StoragePackSubscription> {
        validate_segment("subscription_id", subscription_id)?;
        let response = self
            .auth_request(
                Method::DELETE,
                &format!("{}/{subscription_id}", base(org_id, app_id, collection)),
            )
            .await?
            .send()
            .await?;
        CopepodClient::handle_response_pub(response).await
    }
}
