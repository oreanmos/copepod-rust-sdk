use serde::{Deserialize, Serialize};

/// Optional country evidence sent with a billing country.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingEvidence {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_country: Option<String>,
}

/// Public signup policy needed to decide whether billing can start in an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBillingSignupPolicy {
    pub registration_requires_billing: bool,
    #[serde(default)]
    pub allowed_billing_countries: Vec<String>,
    pub trial_requires_payment_method: bool,
    #[serde(default)]
    pub trial_plan_slug: Option<String>,
    #[serde(default)]
    pub trial_duration_days: Option<i32>,
}

/// A started no-card trial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialStartSummary {
    pub plan_slug: String,
    pub trial_ends_at: String,
}

/// Body for starting the signed-in user's no-card trial.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeTrialStartRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<BillingEvidence>,
}

/// Response of the trial start call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeTrialStartResponse {
    pub trial: TrialStartSummary,
}

/// Body for starting the signed-in user's paid checkout.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeCheckoutRequest {
    pub plan_slug: String,
    /// `month` or `year`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_interval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<BillingEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promo_code: Option<String>,
    /// HTTPS return URL; the server applies its default when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
}

/// Details of a `409 checkout_pending` error from `me_start_checkout`: the
/// earlier open checkout to send the user back to. Read it with
/// `CopepodError::checkout_pending_details`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckoutPending {
    /// The open payment's checkout URL, when the server still has one.
    #[serde(default)]
    pub checkout_url: Option<String>,
    /// RFC 3339 expiry: the payment's Mollie `expiresAt`, or the session's
    /// creation time plus 6 hours.
    pub expires_at: String,
}

/// Payment redirect and billing-period anchor for app-user checkout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeCheckoutResponse {
    pub payment_id: String,
    pub checkout_url: String,
    pub period_starts_at: String,
}

/// Typed body for app registration with country and evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<BillingEvidence>,
}

/// Successful app-user registration: tokens, the created record and the
/// no-card trial the server started, if any.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegisterResponse {
    pub token: String,
    pub refresh_token: String,
    /// The created user record, as the server returns it.
    pub record: serde_json::Value,
    /// Present when registration started a no-card trial.
    #[serde(default)]
    pub trial: Option<TrialStartSummary>,
}
