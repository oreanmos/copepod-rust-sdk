//! Credit ledger shapes (`me/credits*`). One credit is EUR 0.01 of provider
//! cost; all amounts are whole credits.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// An unexpired top-up grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditTopup {
    pub grant_id: String,
    pub remaining: i64,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// `GET me/credits`: this month's allowance and the top-ups behind it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditStatus {
    pub period_start: String,
    pub period_end: String,
    pub allowance: i64,
    pub allowance_used: i64,
    #[serde(default)]
    pub topups: Vec<CreditTopup>,
    /// Allowance left plus unexpired top-ups.
    pub remaining_total: i64,
    /// Share of what was available this month that is left, 0.0 to 1.0.
    pub remaining_ratio: f64,
    /// Credits spent this month by kind (`ai`, `connector:ors`, ...).
    #[serde(default)]
    pub used_by_kind: BTreeMap<String, i64>,
}

/// Body of `POST me/credits/check`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditCheckRequest {
    /// `ai`, `ai:<name>` or `connector:<name>`.
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimate_credits: Option<i64>,
}

/// Answer of `POST me/credits/check` when enough credits are left. When not,
/// the call fails with 402 `credits_exhausted` (see
/// [`crate::CopepodError::credits_exhausted_details`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditCheckResponse {
    pub allowed: bool,
    /// The current balance (allowance left plus unexpired top-ups).
    pub remaining_total: i64,
}

/// Body of `POST me/credits/report`. AI kinds send `cost_micro_eur`;
/// connectors are charged their configured price, and a `credits` value, if
/// sent, must match it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditReportRequest {
    /// Stable id that makes a retry return the first answer.
    pub request_id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_micro_eur: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credits: Option<i64>,
}

/// Answer of `POST me/credits/report`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditReportResponse {
    /// Credits taken for this request (the event cost rounded up).
    pub charged_credits: i64,
    /// The current balance, not the balance right after this request: a replay
    /// returns the first `charged_credits` with today's balance.
    pub remaining_total: i64,
}

/// Details of a `credits_exhausted` error (402 on check, 409 on report).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditsExhausted {
    pub remaining_total: i64,
    /// When the monthly allowance resets (RFC 3339).
    pub resets_at: String,
}
