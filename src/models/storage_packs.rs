//! Storage pack shapes (`me/storage-packs*`).

use serde::{Deserialize, Serialize};

/// A storage pack on sale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePackInfo {
    pub id: String,
    /// Gigabytes one unit adds.
    pub gb: i64,
    /// Price of one unit per month, in minor units (pence or cents).
    pub price_minor: i64,
    pub currency: String,
    /// Always `month`: each pack is its own monthly charge.
    pub interval: String,
}

/// A storage pack subscription the caller owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedStoragePack {
    pub subscription_id: String,
    pub pack_id: String,
    pub quantity: i64,
    /// `pending` (first payment not yet confirmed), `active`, `cancelling` or
    /// `lapsed` (a renewal failed; still listed and still cancellable).
    pub status: String,
    /// End of the paid month; `None` while pending.
    #[serde(default)]
    pub renews_at: Option<String>,
}

/// `GET me/storage-packs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePackCatalog {
    pub storage_packs: Vec<StoragePackInfo>,
    pub owned_storage: Vec<OwnedStoragePack>,
    /// False while the app has no spare storage capacity: subscribing is
    /// refused with `capacity_unavailable`.
    pub capacity_open: bool,
}

/// Body of `POST me/storage-packs/{pack_id}/subscribe`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePackSubscribeRequest {
    /// Units of the pack, 1 to 100.
    pub quantity: i64,
}

/// A storage pack subscription after subscribe or cancel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePackSubscription {
    pub pack_subscription_id: String,
    /// `pending` after subscribe until the first payment is confirmed;
    /// `cancelling` or `cancelled` after a cancel.
    pub status: String,
    /// After a cancel: when the space ends (the paid month's end, or now for a
    /// pack that was never paid). `None` after subscribe.
    #[serde(default)]
    pub ends_at: Option<String>,
}
