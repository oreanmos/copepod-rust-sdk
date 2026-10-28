//! Trust and safety for the app's owner (platform bearer, org admin): the
//! review queue, the decision and report lists, wrong-decision labels, abuse
//! views and suspension. See `docs/trust-safety.md`.

use reqwest::Method;

use super::trust::{app_base, invalid, validate_note, validate_user_id};
use crate::api::spaces::validate_id;
use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{
    AppUserTrust, ContentReportPage, ContentReportQuery, FlaggedAppUsers, ResolveReviewRequest,
    ResolveReviewResponse, RestrictionScope, ReviewAction, ReviewQueuePage, ReviewQueueQuery,
    SuspensionState, TrustDecisionPage, TrustDecisionQuery, WrongDecisionRequest,
    WrongDecisionResponse,
};

/// Longest suspension reason, in characters.
pub const SUSPEND_REASON_MAX_CHARS: usize = 500;

fn limit_param(limit: Option<u32>, query: &mut Vec<(&'static str, String)>) -> Result<()> {
    match limit {
        Some(l) if !(1..=200).contains(&l) => invalid("limit must be between 1 and 200"),
        Some(l) => {
            query.push(("limit", l.to_string()));
            Ok(())
        }
        None => Ok(()),
    }
}

impl CopepodClient {
    /// `GET …/trust/queue` (org admin): open `review` decisions routed to the operator and
    /// content reports. Never carries the submitted text.
    ///
    /// # Errors
    /// `InvalidArgument` for a `limit` outside 1..=200 or an `Unknown` filter; 400
    /// `invalid_trust_filter`.
    pub async fn list_review_queue(
        &self,
        org: &str,
        app: &str,
        q: &ReviewQueueQuery,
    ) -> Result<ReviewQueuePage> {
        let mut query = Vec::new();
        if let Some(status) = q.status {
            query.push(("status", status.as_str().to_string()));
        }
        if let Some(kind) = q.kind {
            query.push(("kind", kind.as_str().to_string()));
        }
        if let Some(cursor) = &q.cursor {
            query.push(("cursor", cursor.clone()));
        }
        limit_param(q.limit, &mut query)?;
        reject_unknown(&query)?;
        let path = format!("{}/trust/queue", app_base(org, app)?);
        let response = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&query)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/trust/queue/{item_id}/resolve` (org admin): keep or remove one item (`td_…`
    /// decision or `cr_…` report). A removal appends a `takedown` to the actions feed, a
    /// keep after a hold a `restore`.
    ///
    /// Returns the server's [`ResolveReviewResponse`] (the spec's `ReviewQueueItem` is not
    /// what the server answers).
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id, an `Unknown` action or a note over 500 characters;
    /// 404 `queue_item_not_found`, 409 `queue_item_resolved`, 400 `invalid_queue_request`.
    pub async fn resolve_review_item(
        &self,
        org: &str,
        app: &str,
        item_id: &str,
        request: &ResolveReviewRequest,
    ) -> Result<ResolveReviewResponse> {
        validate_id("item_id", item_id)?;
        validate_note(request.note.as_deref())?;
        if request.action == ReviewAction::Unknown {
            return invalid("action must be keep or remove");
        }
        let path = format!("{}/trust/queue/{item_id}/resolve", app_base(org, app)?);
        let response = self
            .auth_request(Method::POST, &path)
            .await?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `GET …/trust/decisions` (org admin): the decision log of the last 90 days.
    ///
    /// # Errors
    /// `InvalidArgument` for a `limit` outside 1..=200; 400 `invalid_trust_filter`.
    pub async fn list_trust_decisions(
        &self,
        org: &str,
        app: &str,
        q: &TrustDecisionQuery,
    ) -> Result<TrustDecisionPage> {
        let mut query = Vec::new();
        if let Some(purpose) = q.purpose {
            query.push(("purpose", purpose.as_str().to_string()));
        }
        if let Some(decision) = q.decision {
            query.push(("decision", decision.as_str().to_string()));
        }
        if let Some(engine) = q.engine {
            query.push(("engine", engine.as_str().to_string()));
        }
        if q.degraded_only {
            query.push(("degraded", "true".to_string()));
        }
        if let Some(before) = &q.before {
            query.push(("before", before.clone()));
        }
        limit_param(q.limit, &mut query)?;
        reject_unknown(&query)?;
        let path = format!("{}/trust/decisions", app_base(org, app)?);
        let response = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&query)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/trust/decisions/{decision_id}/wrong` (org admin): label a decision wrong.
    /// For content purposes the answer carries the corrected `outcome` and the `action`
    /// (`restore` or `takedown`) emitted to the actions feed.
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id or a note over 500 characters; 404
    /// `decision_not_found`, 409 `decision_already_wrong` / `decision_in_review`.
    pub async fn mark_decision_wrong(
        &self,
        org: &str,
        app: &str,
        decision_id: &str,
        request: &WrongDecisionRequest,
    ) -> Result<WrongDecisionResponse> {
        validate_id("decision_id", decision_id)?;
        validate_note(request.note.as_deref())?;
        let path = format!(
            "{}/trust/decisions/{decision_id}/wrong",
            app_base(org, app)?
        );
        let response = self
            .auth_request(Method::POST, &path)
            .await?
            .json(request)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `GET …/trust/reports` (org admin): content reports, open ones unless
    /// `q.resolved`. Carries what the reporter wrote in `details`.
    pub async fn list_content_reports(
        &self,
        org: &str,
        app: &str,
        q: &ContentReportQuery,
    ) -> Result<ContentReportPage> {
        let mut query = vec![(
            "status",
            if q.resolved { "resolved" } else { "open" }.to_string(),
        )];
        if let Some(cursor) = &q.cursor {
            query.push(("cursor", cursor.clone()));
        }
        limit_param(q.limit, &mut query)?;
        let path = format!("{}/trust/reports", app_base(org, app)?);
        let response = self
            .auth_request(Method::GET, &path)
            .await?
            .query(&query)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `GET …/app-users/{user_id}/trust` (org admin): a user's scores, signals, active
    /// restrictions and suspension.
    pub async fn get_app_user_trust(
        &self,
        org: &str,
        app: &str,
        user_id: &str,
    ) -> Result<AppUserTrust> {
        validate_user_id(user_id)?;
        let path = format!("{}/app-users/{user_id}/trust", app_base(org, app)?);
        self.get(&path).await
    }

    /// `GET …/trust/users` (org admin): users with a signal in the last 30 days or an
    /// active restriction (at most 100).
    pub async fn list_flagged_app_users(&self, org: &str, app: &str) -> Result<FlaggedAppUsers> {
        let path = format!("{}/trust/users", app_base(org, app)?);
        self.get(&path).await
    }

    /// `POST …/app-users/{user_id}/restrictions/{scope}/lift` (org admin): end an
    /// automatic restriction early and reset the scope's score. A suspension is lifted
    /// with [`Self::unsuspend_app_user`]; `Account` is refused before any request.
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id or the `Account`/`Unknown` scope; 404
    /// `restriction_not_found` (no active restriction in that scope; see
    /// `CopepodError::is_restriction_not_found`).
    pub async fn lift_app_user_restriction(
        &self,
        org: &str,
        app: &str,
        user_id: &str,
        scope: RestrictionScope,
    ) -> Result<()> {
        validate_user_id(user_id)?;
        if matches!(scope, RestrictionScope::Account | RestrictionScope::Unknown) {
            return invalid("scope must be hosted_ai, public_posting or invites; use unsuspend_app_user for a suspension");
        }
        let path = format!(
            "{}/app-users/{user_id}/restrictions/{}/lift",
            app_base(org, app)?,
            scope.as_str()
        );
        let response = self.auth_request(Method::POST, &path).await?.send().await?;
        Self::handle_empty_response(response).await
    }

    /// `POST …/app-users/{user_id}/suspend` (org admin): block the user's sign-in, refresh,
    /// existing tokens, records and realtime. Appends `suspend` to the actions feed.
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id or a reason over 500 characters; 404
    /// `app_user_not_found`, 409 `already_suspended`, 503 while the trust tables upgrade.
    pub async fn suspend_app_user(
        &self,
        org: &str,
        app: &str,
        user_id: &str,
        reason: &str,
    ) -> Result<SuspensionState> {
        validate_user_id(user_id)?;
        if reason.chars().count() > SUSPEND_REASON_MAX_CHARS {
            return invalid(format!(
                "reason must be at most {SUSPEND_REASON_MAX_CHARS} characters"
            ));
        }
        let path = format!("{}/app-users/{user_id}/suspend", app_base(org, app)?);
        let response = self
            .auth_request(Method::POST, &path)
            .await?
            .json(&serde_json::json!({ "reason": reason }))
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// `POST …/app-users/{user_id}/unsuspend` (org admin).
    ///
    /// # Errors
    /// `InvalidArgument` for an empty id; 404 `app_user_not_found`, 409 `not_suspended`.
    pub async fn unsuspend_app_user(
        &self,
        org: &str,
        app: &str,
        user_id: &str,
    ) -> Result<SuspensionState> {
        validate_user_id(user_id)?;
        let path = format!("{}/app-users/{user_id}/unsuspend", app_base(org, app)?);
        let response = self.auth_request(Method::POST, &path).await?.send().await?;
        Self::handle_response_pub(response).await
    }
}

/// A filter that deserialised to `Unknown` would send `unknown`, which the server rejects:
/// fail before the network instead.
fn reject_unknown(query: &[(&'static str, String)]) -> Result<()> {
    if query.iter().any(|(_, v)| v == "unknown") {
        return invalid("a filter has an unknown value");
    }
    Ok(())
}
