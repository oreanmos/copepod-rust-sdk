use reqwest::Method;

use super::email::{looks_like_email_address, validate_app_scope};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    ActiveAnnouncement, ActiveAnnouncementsResponse, CreateLeadRequest, CreateLeadResponse,
};

const MAX_EMAIL_BYTES: usize = 320;
const MAX_CONSENT_CHARS: usize = 500;

impl CopepodClient {
    /// Add a visitor to an audience list (double opt-in) via
    /// `POST /api/platform/orgs/{org}/apps/{app}/leads`.
    ///
    /// Requires an API key with scope `leads:write`; no Bearer token is sent.
    /// Arguments are validated before any request and rejected with
    /// [`CopepodError::InvalidArgument`]. Every well-formed request answers
    /// `accepted: true` whether the address is new, pending, confirmed or
    /// unsubscribed, so show the same message in every case. Server errors
    /// arrive as [`CopepodError::Api`] with the server message: 422
    /// `turnstile_failed`, 429 `rate_limited`, 503 `turnstile_unavailable` or
    /// `audience_upgrading`. See `docs/audience.md`.
    pub async fn create_lead(
        &self,
        org_id: &str,
        app_id: &str,
        lead: &CreateLeadRequest,
    ) -> Result<CreateLeadResponse> {
        validate_app_scope(org_id, app_id)?;
        validate_lead(lead)?;
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/leads");
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(lead)
            .send()
            .await?;
        Self::handle_response_pub(response).await
    }

    /// List the announcements currently active for an app via
    /// `GET /api/platform/orgs/{org}/apps/{app}/announcements/active`.
    ///
    /// Requires an API key with scope `announcements:read`; no Bearer token is
    /// sent. Returns at most 20 items, most severe first. The server marks the
    /// response `Cache-Control: private, max-age=60`: call at most once a
    /// minute per process and cache the result.
    pub async fn list_active_announcements(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<Vec<ActiveAnnouncement>> {
        validate_app_scope(org_id, app_id)?;
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/announcements/active");
        let response = self.api_key_request(Method::GET, &path)?.send().await?;
        let body: ActiveAnnouncementsResponse = Self::handle_response_pub(response).await?;
        Ok(body.announcements)
    }
}

fn invalid(message: &str) -> CopepodError {
    CopepodError::InvalidArgument(message.into())
}

fn validate_lead(lead: &CreateLeadRequest) -> Result<()> {
    let key = lead.list_key.as_bytes();
    let key_ok = (1..=40).contains(&key.len())
        && key[0].is_ascii_lowercase()
        && key
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_');
    if !key_ok {
        return Err(invalid("list key must match ^[a-z][a-z0-9_]{0,39}$"));
    }
    let email = lead.email.trim();
    if email.len() > MAX_EMAIL_BYTES
        || email.chars().any(char::is_control)
        || !looks_like_email_address(email)
    {
        return Err(invalid(
            "lead email must be a valid address of at most 320 bytes",
        ));
    }
    let consent_chars = lead.consent_text.trim().chars().count();
    if !(1..=MAX_CONSENT_CHARS).contains(&consent_chars) {
        return Err(invalid("consent text must contain 1-500 characters"));
    }
    if let Some(country) = &lead.country {
        let b = country.as_bytes();
        if b.len() != 2 || !b.iter().all(u8::is_ascii_uppercase) {
            return Err(invalid(
                "country must be two upper-case letters (ISO 3166-1)",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_lead;
    use crate::models::CreateLeadRequest;

    fn lead() -> CreateLeadRequest {
        CreateLeadRequest {
            list_key: "newsletter".into(),
            email: "a@b.co".into(),
            consent_text: "ok".into(),
            ..Default::default()
        }
    }

    #[test]
    fn accepts_a_minimal_lead_and_rejects_bad_fields() {
        assert!(validate_lead(&lead()).is_ok());
        for bad in [
            CreateLeadRequest {
                list_key: "Bad".into(),
                ..lead()
            },
            CreateLeadRequest {
                list_key: "1x".into(),
                ..lead()
            },
            CreateLeadRequest {
                email: "nope".into(),
                ..lead()
            },
            CreateLeadRequest {
                consent_text: "  ".into(),
                ..lead()
            },
            CreateLeadRequest {
                consent_text: "x".repeat(501),
                ..lead()
            },
            CreateLeadRequest {
                country: Some("gb".into()),
                ..lead()
            },
        ] {
            assert!(validate_lead(&bad).is_err());
        }
    }
}
