//! Laya `decide`, decide outcome and donations (AI4).
//!
//! Callers treat every failure of these routes as "no answer", so no separate "unavailable" value
//! is returned. A node older than the routes answers 404 with an empty body, which arrives as
//! `CopepodError::Api { status: 404, code: None, message: "Unknown error" }`;
//! [`CopepodError::is_route_missing`] recognises it (a donation UI can hide its toggle). See
//! `docs/ai-laya.md`.

use reqwest::Method;

use crate::api::spaces::validate_id;
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    DecideOutcome, DecideRequest, DecideResponse, Donation, DonationConsent, DonationConsentResult,
    DonationCreated, DonationStatus, DonationsDeleted, OutcomeSource, DONATION_POLICY_VERSION,
};

/// Longest ad-hoc `state` and donated `text`, in characters (as the server counts them).
const MAX_STATE_CHARS: usize = 4_000;
/// Most questions in an ad-hoc call (one service batch).
const MAX_ADHOC_QUESTIONS: usize = 3;
/// Longest donation `language` tag, in bytes.
const MAX_LANGUAGE_BYTES: usize = 16;

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(CopepodError::InvalidArgument(message.into()))
}

/// Checks the server makes on a decide body, so a bad call fails before the network.
fn validate_decide(request: &DecideRequest) -> Result<()> {
    match request {
        DecideRequest::Task(task) => {
            if task.task.is_empty() {
                return invalid("task must not be empty");
            }
        }
        DecideRequest::AdHoc(adhoc) => {
            let chars = adhoc.state.chars().count();
            if chars == 0 || chars > MAX_STATE_CHARS {
                return invalid("ad-hoc state must be 1 to 4000 characters");
            }
            if adhoc.questions.is_empty() || adhoc.questions.len() > MAX_ADHOC_QUESTIONS {
                return invalid("ad-hoc questions must be 1 to 3 entries");
            }
            let mut seen = std::collections::HashSet::new();
            for question in &adhoc.questions {
                if question.kind != "choice" {
                    return invalid("ad-hoc questions must have type \"choice\"");
                }
                if !seen.insert(question.id.as_str()) {
                    return invalid("ad-hoc question ids must be unique");
                }
            }
        }
    }
    Ok(())
}

fn validate_labels(labels: &std::collections::BTreeMap<String, String>) -> Result<()> {
    if labels.is_empty() {
        return invalid("labels must name at least one question");
    }
    if labels.iter().any(|(q, o)| q.is_empty() || o.is_empty()) {
        return invalid("labels must not contain an empty question or option");
    }
    Ok(())
}

fn validate_donation(donation: &Donation) -> Result<()> {
    if donation.task.is_empty() {
        return invalid("task must not be empty");
    }
    let chars = donation.text.chars().count();
    if donation.text.trim().is_empty() || chars > MAX_STATE_CHARS {
        return invalid("text must be 1 to 4000 characters");
    }
    validate_labels(&donation.labels)?;
    if donation
        .language
        .as_ref()
        .is_some_and(|l| l.is_empty() || l.len() > MAX_LANGUAGE_BYTES)
    {
        return invalid("language must be a short tag (1 to 16 bytes)");
    }
    Ok(())
}

fn user_base(org_id: &str, app_id: &str, collection: &str) -> Result<String> {
    validate_id("org_id", org_id)?;
    validate_id("app_id", app_id)?;
    validate_id("collection", collection)?;
    Ok(format!(
        "api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/ai"
    ))
}

fn key_base(org_id: &str, app_id: &str) -> Result<String> {
    validate_id("org_id", org_id)?;
    validate_id("app_id", app_id)?;
    Ok(format!("api/platform/orgs/{org_id}/apps/{app_id}/ai"))
}

impl CopepodClient {
    /// `POST …/auth/{collection}/me/ai/decide` (bearer): ask Laya a registered task's questions.
    ///
    /// **Act on an answer only when its `confident` is true**; otherwise use the task's
    /// escalation. Decide is app overhead and never charges the user's credits. Ad-hoc requests
    /// are refused here with `InvalidArgument` (the server answers 403 `laya_adhoc_forbidden`):
    /// use [`Self::ai_decide_with_api_key`].
    ///
    /// # Errors
    /// `InvalidArgument` for empty ids, an empty task; 400
    /// `laya_invalid_request`, 404 `laya_task_unknown`, 503 `laya_unavailable` (treat as "no
    /// answer") and 403 `ai_paused_for_review` as `CopepodError::Api`.
    pub async fn ai_decide(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        request: &DecideRequest,
    ) -> Result<DecideResponse> {
        let base = user_base(org_id, app_id, collection)?;
        if matches!(request, DecideRequest::AdHoc(_)) {
            return invalid(
                "ad-hoc decide needs an API key (the server answers 403 laya_adhoc_forbidden to a bearer); use ai_decide_with_api_key",
            );
        }
        validate_decide(request)?;
        let resp = self
            .auth_request(Method::POST, &format!("{base}/decide"))
            .await?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `POST /api/platform/orgs/{org}/apps/{app}/ai/decide` (`X-API-Key`, scope `ai:decide`).
    ///
    /// Sends `X-API-Key` only, never `Authorization`. `user_id` is sent as `X-User-Id` for
    /// attribution. Takes a task request or an ad-hoc one (at most 3 questions, state at most
    /// 4,000 characters). An ad-hoc answer has `mode: "adhoc"`, no `decision_id` and
    /// `confident: false`; apply your own threshold to `confidence`.
    ///
    /// # Errors
    /// `Auth` when the client has no API key; otherwise as [`Self::ai_decide`].
    pub async fn ai_decide_with_api_key(
        &self,
        org_id: &str,
        app_id: &str,
        request: &DecideRequest,
        user_id: Option<&str>,
    ) -> Result<DecideResponse> {
        let base = key_base(org_id, app_id)?;
        validate_decide(request)?;
        // Intentional: an empty `X-User-Id` is never a valid attribution, so refuse it here.
        if user_id.is_some_and(str::is_empty) {
            return invalid("user_id must not be empty");
        }
        let mut http = self
            .api_key_request(Method::POST, &format!("{base}/decide"))?
            .json(request);
        if let Some(user_id) = user_id {
            http = http.header("X-User-Id", user_id);
        }
        Self::handle_response_pub(http.send().await?).await
    }

    /// `POST …/me/ai/decide/{decision_id}/outcome` (bearer): report the user's own label.
    ///
    /// This route always records `source: user`, so `outcome.source` must be
    /// [`OutcomeSource::User`] (`tier2` and `tier3` are API-key only). The text is never sent.
    /// The first label per (decision, source) wins; a repeat is accepted and changes nothing.
    ///
    /// # Errors
    /// `InvalidArgument` for empty ids, no labels or a `tier2`/`tier3` source; 404
    /// `laya_decision_unknown`, 400 `laya_invalid_label`, 503 `laya_upgrading` as `Api`.
    pub async fn ai_decide_outcome(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        decision_id: &str,
        outcome: &DecideOutcome,
    ) -> Result<()> {
        let base = user_base(org_id, app_id, collection)?;
        validate_id("decision_id", decision_id)?;
        if outcome.source != OutcomeSource::User {
            return invalid(
                "the app-user outcome route records source `user`; tier2 and tier3 need ai_decide_outcome_with_api_key",
            );
        }
        validate_labels(&outcome.labels)?;
        let resp = self
            .auth_request(
                Method::POST,
                &format!("{base}/decide/{decision_id}/outcome"),
            )
            .await?
            .json(outcome)
            .send()
            .await?;
        Self::handle_empty_response(resp).await
    }

    /// `POST /api/platform/orgs/{org}/apps/{app}/ai/decide/{decision_id}/outcome`
    /// (`X-API-Key`, scope `ai:decide`; no `Authorization`): report a `tier2`, `tier3` or `user` label.
    ///
    /// # Errors
    /// `Auth` when the client has no API key; otherwise as [`Self::ai_decide_outcome`].
    pub async fn ai_decide_outcome_with_api_key(
        &self,
        org_id: &str,
        app_id: &str,
        decision_id: &str,
        outcome: &DecideOutcome,
    ) -> Result<()> {
        let base = key_base(org_id, app_id)?;
        validate_id("decision_id", decision_id)?;
        validate_labels(&outcome.labels)?;
        let resp = self
            .api_key_request(
                Method::POST,
                &format!("{base}/decide/{decision_id}/outcome"),
            )?
            .json(outcome)
            .send()
            .await?;
        Self::handle_empty_response(resp).await
    }

    /// `GET …/me/ai/donations` (bearer): whether donating is on, how many donations, the oldest.
    pub async fn ai_donations_status(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<DonationStatus> {
        let base = user_base(org_id, app_id, collection)?;
        let resp = self
            .auth_request(Method::GET, &format!("{base}/donations"))
            .await?
            .send()
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `PUT …/me/ai/donations/consent` (bearer): opt in (with the policy version, see
    /// [`DonationConsent::grant`]) or opt out, which **deletes every donation**.
    ///
    /// # Errors
    /// `InvalidArgument` for empty ids or a grant without `policy_version`; 400
    /// `laya_policy_version_unknown`, 503 `laya_upgrading` as `Api`.
    pub async fn ai_set_donation_consent(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        consent: &DonationConsent,
    ) -> Result<DonationConsentResult> {
        let base = user_base(org_id, app_id, collection)?;
        if consent.enabled && consent.policy_version.as_deref().unwrap_or("").is_empty() {
            return invalid(format!(
                "granting consent needs a policy_version (currently \"{DONATION_POLICY_VERSION}\")"
            ));
        }
        let resp = self
            .auth_request(Method::PUT, &format!("{base}/donations/consent"))
            .await?
            .json(consent)
            .send()
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `POST …/me/ai/donations` (bearer): donate one text with its labels.
    ///
    /// # Errors
    /// `InvalidArgument` for empty ids, an empty or over-4,000-character text, no labels or a bad
    /// language tag; 403 `laya_donation_consent_required`, 400 `laya_task_not_donatable` /
    /// `laya_invalid_label`, 429 `laya_donation_quota`, 503 `laya_upgrading` as `Api`.
    pub async fn ai_donate(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        donation: &Donation,
    ) -> Result<DonationCreated> {
        let base = user_base(org_id, app_id, collection)?;
        validate_donation(donation)?;
        let resp = self
            .auth_request(Method::POST, &format!("{base}/donations"))
            .await?
            .json(donation)
            .send()
            .await?;
        Self::handle_response_pub(resp).await
    }

    /// `DELETE …/me/ai/donations` (bearer): delete every donation; the consent stays.
    pub async fn ai_delete_donations(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<DonationsDeleted> {
        let base = user_base(org_id, app_id, collection)?;
        let resp = self
            .auth_request(Method::DELETE, &format!("{base}/donations"))
            .await?
            .send()
            .await?;
        Self::handle_response_pub(resp).await
    }
}
