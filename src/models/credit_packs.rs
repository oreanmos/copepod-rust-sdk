//! One-off credit pack shapes (`me/credits/packs*`, `me/credits/purchases*`).

use serde::{Deserialize, Serialize};

/// A credit pack on sale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditPackInfo {
    pub id: String,
    pub credits: i64,
    /// Price in minor units (pence or cents).
    pub price_minor: i64,
    pub currency: String,
}

/// `GET me/credits/packs`: the packs on sale, cheapest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditPackCatalog {
    pub packs: Vec<CreditPackInfo>,
}

/// Body of `POST me/credits/packs/{pack_id}/purchase`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditPackPurchaseRequest {
    /// Where the provider sends the buyer afterwards (HTTPS, allowed by the app).
    pub redirect_url: String,
}

/// A pack purchase, from `POST .../purchase` and `GET me/credits/purchases*`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditPackPurchase {
    pub purchase_id: String,
    pub pack_id: String,
    pub credits: i64,
    /// `pending`, `paid`, `failed`, `expired`, `cancelled` or `revoked`.
    pub status: String,
    #[serde(default)]
    pub checkout_url: Option<String>,
    pub created: String,
    #[serde(default)]
    pub paid_at: Option<String>,
}

/// `GET me/credits/purchases`: the caller's purchases, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditPackPurchaseList {
    pub items: Vec<CreditPackPurchase>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
