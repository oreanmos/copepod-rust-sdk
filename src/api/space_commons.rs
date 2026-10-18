//! Commons moderation: contributors suggest records or changes, moderators
//! and owners accept or reject them or take a record down. Every route needs
//! an app-user bearer token.
//!
//! Shared errors: 404 `space_not_found`, 400 `not_commons`, 403
//! `insufficient_role`, 503 `commons_upgrading` while a rolling upgrade holds
//! commons writes.

use serde_json::{json, Value};

use super::spaces::{base, validate_id};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    SpaceSuggestion, SuggestionAccepted, SuggestionCreate, SuggestionListOptions, SuggestionPage,
    TakedownResult,
};

const STATUSES: [&str; 4] = ["pending", "accepted", "rejected", "withdrawn"];
const MAX_REASON_CHARS: usize = 1000;
const MAX_PROPOSED_BYTES: usize = 16 * 1024;

fn validate_reason(what: &str, reason: &str, required: bool) -> Result<()> {
    let reason = reason.trim();
    if (required && reason.is_empty())
        || reason.chars().count() > MAX_REASON_CHARS
        || reason.chars().any(|c| c.is_control() && c != '\n')
    {
        return Err(CopepodError::InvalidArgument(format!(
            "{what} must be {}at most {MAX_REASON_CHARS} characters without control characters",
            if required { "non-empty, " } else { "" }
        )));
    }
    Ok(())
}

impl CopepodClient {
    /// Propose a new record, or with `record_id` a change, to a commons
    /// (`POST spaces/{id}/suggestions`, contributor or above).
    ///
    /// `proposed` must be an object of at most 16 KB naming schema fields of
    /// a space-scoped collection (not `id`, `created`, `updated`,
    /// `created_by`, owner, read-only or hidden fields: 400). Errors: 429
    /// `suggestion_rate_limited` (20 an hour per commons, 60 across the app,
    /// 100 waiting).
    pub async fn create_space_suggestion(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        body: &SuggestionCreate,
    ) -> Result<SpaceSuggestion> {
        validate_id("space_id", space_id)?;
        validate_id("collection", &body.collection)?;
        if let Some(record_id) = &body.record_id {
            validate_id("record_id", record_id)?;
        }
        validate_reason("reason", &body.reason, true)?;
        if body.proposed.as_object().is_none_or(|o| o.is_empty()) {
            return Err(CopepodError::InvalidArgument(
                "proposed must be a non-empty JSON object".into(),
            ));
        }
        if body.proposed.to_string().len() > MAX_PROPOSED_BYTES {
            return Err(CopepodError::InvalidArgument(
                "proposed must be at most 16 KB".into(),
            ));
        }
        self.post(
            &format!("{}/spaces/{space_id}/suggestions", base(org_id, app_id)),
            body,
        )
        .await
    }

    /// List a commons' suggestions (`GET spaces/{id}/suggestions`).
    /// Moderators and owners see all, contributors their own.
    ///
    /// Pages are cursor-based: pass [`SuggestionPage::next`] as
    /// [`SuggestionListOptions::after`] until it is `None`.
    pub async fn list_space_suggestions(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        options: &SuggestionListOptions,
    ) -> Result<SuggestionPage> {
        validate_id("space_id", space_id)?;
        if let Some(status) = &options.status {
            if !STATUSES.contains(&status.as_str()) {
                return Err(CopepodError::InvalidArgument(
                    "status must be pending, accepted, rejected or withdrawn".into(),
                ));
            }
        }
        if options.limit.is_some_and(|l| !(1..=200).contains(&l)) {
            return Err(CopepodError::InvalidArgument(
                "limit must be between 1 and 200".into(),
            ));
        }
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(v) = &options.status {
            params.push(("status", v.clone()));
        }
        if let Some(v) = options.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = &options.after {
            params.push(("after", v.clone()));
        }
        let path = format!("{}/spaces/{space_id}/suggestions", base(org_id, app_id));
        let resp = self
            .auth_request(reqwest::Method::GET, &path)
            .await?
            .query(&params)
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Accept a suggestion and apply it to the commons
    /// (`POST .../suggestions/{id}/accept`, moderator or owner).
    ///
    /// The collection's create or update rule is evaluated as the accepting
    /// moderator. Errors: 409 `suggestion_decided`, 409 `revision_conflict`
    /// (the record moved on since the proposal; see
    /// [`CopepodError::revision_conflict_details`]), 403 when the rule
    /// refuses (the suggestion stays pending), 404 `suggestion_not_found`.
    pub async fn accept_space_suggestion(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        suggestion_id: &str,
        note: Option<&str>,
    ) -> Result<SuggestionAccepted> {
        let path = decision_path(org_id, app_id, space_id, suggestion_id, "accept", note)?;
        self.post(&path, &note_body(note)).await
    }

    /// Reject a suggestion (`POST .../suggestions/{id}/reject`, moderator or
    /// owner). Returns the decided suggestion; 409 `suggestion_decided` when
    /// it was decided before.
    pub async fn reject_space_suggestion(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        suggestion_id: &str,
        note: Option<&str>,
    ) -> Result<SpaceSuggestion> {
        let path = decision_path(org_id, app_id, space_id, suggestion_id, "reject", note)?;
        self.post(&path, &note_body(note)).await
    }

    /// Take a record down from a commons
    /// (`POST spaces/{id}/records/{collection}/{record_id}/takedown`,
    /// moderator or owner).
    ///
    /// Removes the record, its journal entries, its original files and what
    /// suggestions about it proposed. Repeating it after a failure is safe.
    /// `reason` is required (1000 characters) and goes to the audit trail.
    pub async fn takedown_space_record(
        &self,
        org_id: &str,
        app_id: &str,
        space_id: &str,
        collection: &str,
        record_id: &str,
        reason: &str,
    ) -> Result<TakedownResult> {
        validate_id("space_id", space_id)?;
        validate_id("collection", collection)?;
        validate_id("record_id", record_id)?;
        validate_reason("reason", reason, true)?;
        self.post(
            &format!(
                "{}/spaces/{space_id}/records/{collection}/{record_id}/takedown",
                base(org_id, app_id)
            ),
            &json!({ "reason": reason }),
        )
        .await
    }
}

fn decision_path(
    org_id: &str,
    app_id: &str,
    space_id: &str,
    suggestion_id: &str,
    verb: &str,
    note: Option<&str>,
) -> Result<String> {
    validate_id("space_id", space_id)?;
    validate_id("suggestion_id", suggestion_id)?;
    validate_reason("note", note.unwrap_or_default(), false)?;
    Ok(format!(
        "{}/spaces/{space_id}/suggestions/{suggestion_id}/{verb}",
        base(org_id, app_id)
    ))
}

fn note_body(note: Option<&str>) -> Value {
    match note {
        Some(note) => json!({ "note": note }),
        None => json!({}),
    }
}
