//! Trust and safety for the app: `moderate`, decision outcomes, content
//! reports, the actions feed, abuse signals and the user's own restrictions
//! (AI3). API-key methods send `X-API-Key` only and never `Authorization`;
//! the `*_as_user` and `my_restrictions` methods use the app-user bearer.
//! Org-admin methods are in [`super::trust_admin`]. See `docs/trust-safety.md`.

use reqwest::Method;

use crate::api::spaces::validate_id;
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    AbuseSignalKind, AbuseSignalReceipt, AbuseSignalRequest, ContentReportReceipt,
    ContentReportRequest, ModerateRequest, ModerateResponse, ModerationOutcomeRequest,
    ModerationOutcomeResponse, ModerationPurpose, MyRestrictions, ReportCategory, TrustActionPage,
};

/// Longest `text` of a check, in characters.
pub const MODERATE_TEXT_MAX_CHARS: usize = 20_000;
/// Longest report `details`, in characters.
pub const REPORT_DETAILS_MAX_CHARS: usize = 4_000;
/// Longest outcome or resolve note, in characters.
pub const TRUST_NOTE_MAX_CHARS: usize = 500;
/// Most entries one actions feed page returns.
pub const TRUST_ACTIONS_MAX_LIMIT: u32 = 100;

pub(crate) fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(CopepodError::InvalidArgument(message.into()))
}

pub(crate) fn app_base(org: &str, app: &str) -> Result<String> {
    validate_id("org_id", org)?;
    validate_id("app_id", app)?;
    Ok(format!("api/platform/orgs/{org}/apps/{app}"))
}

/// `kind:id` with `kind` 1-40 of `a-z_` and `id` 1-160 of `A-Za-z0-9._:-` (as the server checks).
pub(crate) fn validate_subject_ref(subject_ref: &str) -> Result<()> {
    let ok = subject_ref.split_once(':').is_some_and(|(kind, id)| {
        (1..=40).contains(&kind.len())
            && kind.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
            && (1..=160).contains(&id.len())
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
    });
    if ok {
        Ok(())
    } else {
        invalid("subject_ref must look like 'kind:id' (kind a-z_, id letters, digits and . _ : -)")
    }
}

pub(crate) fn validate_user_id(user_id: &str) -> Result<()> {
    if user_id.trim().is_empty() {
        return invalid("user_id must not be empty");
    }
    validate_id("user_id", user_id)
}

pub(crate) fn validate_note(note: Option<&str>) -> Result<()> {
    if note.is_some_and(|n| n.chars().count() > TRUST_NOTE_MAX_CHARS) {
        return invalid(format!(
            "note must be at most {TRUST_NOTE_MAX_CHARS} characters"
        ));
    }
    Ok(())
}

fn validate_moderate(request: &ModerateRequest) -> Result<()> {
    if request.purpose == ModerationPurpose::Unknown {
        return invalid("purpose is not a known moderation purpose");
    }
    validate_subject_ref(&request.subject_ref)?;
    if request.text.chars().count() > MODERATE_TEXT_MAX_CHARS {
        return invalid(format!(
            "text must be at most {MODERATE_TEXT_MAX_CHARS} characters"
        ));
    }
    if request.actor.user_id.as_deref().is_some_and(str::is_empty) {
        return invalid("actor.user_id must not be empty");
    }
    Ok(())
}

fn validate_report(request: &ContentReportRequest) -> Result<()> {
    validate_subject_ref(&request.subject_ref)?;
    if request.category == ReportCategory::Unknown {
        return invalid("category is not a known report category");
    }
    if request.details.chars().count() > REPORT_DETAILS_MAX_CHARS {
        return invalid(format!(
            "details must be at most {REPORT_DETAILS_MAX_CHARS} characters"
        ));
    }
    let hash = &request.reporter_ip_hash;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return invalid("reporter_ip_hash must be 64 hex characters");
    }
    if request
        .subject_owner_user_id
        .as_deref()
        .is_some_and(str::is_empty)
    {
        return invalid("subject_owner_user_id must not be empty");
    }
    Ok(())
}

fn validate_signal(request: &AbuseSignalRequest) -> Result<()> {
    validate_user_id(&request.user_id)?;
    if matches!(
        request.kind,
        AbuseSignalKind::Unknown | AbuseSignalKind::ModerationBlock | AbuseSignalKind::ReportUpheld
    ) {
        return invalid("kind must be one the app reports: chat_jailbreak, chat_harm, rate_limit_hammering or invite_cap");
    }
    if request.evidence_ref.chars().count() > 200
        || !request
            .evidence_ref
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'/' | b'-'))
    {
        return invalid(
            "evidence_ref must be at most 200 characters of letters, digits and _ . : / -",
        );
    }
    Ok(())
}

impl CopepodClient {
    /// `POST …/ai/moderate` (`X-API-Key`, scope `ai:moderate`): check text before it is shown.
    ///
    /// A provider outage never fails the call: the answer has `degraded: true` and the
    /// purpose's fail mode. Treat an `Err` as the same and apply your own fail mode.
    /// `reviewer: Operator` sends a `review` decision to the Copepod operator queue.
    ///
    /// # Errors
    /// `InvalidArgument` for a bad purpose, `subject_ref` or `text` over 20,000 characters;
    /// `Auth` without an API key; 400 `invalid_moderation_request`, 429
    /// `moderation_rate_limited`, 503 while a rolling upgrade holds the trust tables.
    pub async fn moderate(
        &self,
        org: &str,
        app: &str,
        request: &ModerateRequest,
    ) -> Result<ModerateResponse> {
        validate_moderate(request)?;
        let path = format!("{}/ai/moderate", app_base(org, app)?);
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/auth/{collection}/me/ai/moderate` (app-user bearer): check the signed-in
    /// user's own text. The server uses the token's user as the actor and forces
    /// `reviewer: App`, ignoring both fields of the request.
    ///
    /// # Errors
    /// As [`Self::moderate`], plus 403 `account_suspended`.
    pub async fn moderate_as_user(
        &self,
        org: &str,
        app: &str,
        collection: &str,
        request: &ModerateRequest,
    ) -> Result<ModerateResponse> {
        validate_moderate(request)?;
        validate_id("collection", collection)?;
        let path = format!("{}/auth/{collection}/me/ai/moderate", app_base(org, app)?);
        let response = self
            .auth_request(Method::POST, &path)
            .await?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/trust/decisions/{decision_id}/outcome` (`X-API-Key`, scope `ai:moderate`):
    /// tell Copepod what the app's owner did with a held item. Idempotent; the app's last
    /// write wins, but an operator's resolution is never overwritten.
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id, `Unknown` outcome or a note over 500 characters;
    /// 404 `decision_not_found`, 400 `invalid_outcome`.
    pub async fn record_moderation_outcome(
        &self,
        org: &str,
        app: &str,
        decision_id: &str,
        request: &ModerationOutcomeRequest,
    ) -> Result<ModerationOutcomeResponse> {
        validate_id("decision_id", decision_id)?;
        validate_note(request.note.as_deref())?;
        if request.outcome == crate::models::ModerationOutcome::Unknown {
            return invalid("outcome must be approved or removed");
        }
        let path = format!(
            "{}/trust/decisions/{decision_id}/outcome",
            app_base(org, app)?
        );
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/reports` (`X-API-Key`, scope `trust:write`): file a content report as the
    /// app's server. The reporter is identified by `reporter_ip_hash`; a repeat of the same
    /// reporter and subject within a day answers `duplicate: true`.
    ///
    /// # Errors
    /// `InvalidArgument` for a bad `subject_ref`, `details` over 4,000 characters or a
    /// `reporter_ip_hash` that is not 64 hex characters; 400 `invalid_report`, 429
    /// `report_rate_limited` (see [`CopepodError::is_report_rate_limited`]).
    pub async fn submit_content_report(
        &self,
        org: &str,
        app: &str,
        request: &ContentReportRequest,
    ) -> Result<ContentReportReceipt> {
        validate_report(request)?;
        let path = format!("{}/reports", app_base(org, app)?);
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `GET …/trust/actions` (`X-API-Key`, scope `trust:read`): the feed of takedowns,
    /// restores, restrictions and suspensions the app applies. Entries come oldest first;
    /// pass the previous page's `next_after` as `after`. `limit` is 1..=100 (default 100).
    ///
    /// # Errors
    /// `InvalidArgument` for a `limit` of 0 or over 100 or a negative `after`.
    pub async fn list_trust_actions(
        &self,
        org: &str,
        app: &str,
        after: Option<i64>,
        limit: u32,
    ) -> Result<TrustActionPage> {
        if !(1..=TRUST_ACTIONS_MAX_LIMIT).contains(&limit) {
            return invalid(format!(
                "limit must be between 1 and {TRUST_ACTIONS_MAX_LIMIT}"
            ));
        }
        if after.is_some_and(|a| a < 0) {
            return invalid("after must not be negative");
        }
        let path = format!("{}/trust/actions", app_base(org, app)?);
        let mut query = vec![("limit", limit.to_string())];
        if let Some(after) = after {
            query.push(("after", after.to_string()));
        }
        let response = self
            .api_key_request(Method::GET, &path)?
            .query(&query)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/trust/signals` (`X-API-Key`, scope `trust:write`): report abuse only the app
    /// can see (`chat_jailbreak`, `chat_harm`, `rate_limit_hammering`, `invite_cap`).
    /// `moderation_block` and `report_upheld` are recorded by Copepod and are refused here.
    /// The receipt's `restriction` is set when this signal tipped the score over 2.5.
    ///
    /// # Errors
    /// `InvalidArgument` for an empty `user_id`, an `Unknown` kind or a bad `evidence_ref`;
    /// 400 `invalid_signal`, 404 `app_user_not_found`.
    pub async fn record_abuse_signal(
        &self,
        org: &str,
        app: &str,
        request: &AbuseSignalRequest,
    ) -> Result<AbuseSignalReceipt> {
        validate_signal(request)?;
        let path = format!("{}/trust/signals", app_base(org, app)?);
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `GET …/auth/{collection}/me/restrictions` (app-user bearer): the signed-in user's
    /// active restrictions and whether the account is suspended.
    pub async fn my_restrictions(
        &self,
        org: &str,
        app: &str,
        collection: &str,
    ) -> Result<MyRestrictions> {
        validate_id("collection", collection)?;
        let path = format!("{}/auth/{collection}/me/restrictions", app_base(org, app)?);
        self.get(&path).await
    }
}
