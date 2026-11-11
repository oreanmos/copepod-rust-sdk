use std::net::IpAddr;

use reqwest::Method;

use super::email::{looks_like_email_address, validate_app_scope};
use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::audience::validate_consent_text;
use crate::models::{
    ActiveAnnouncement, ActiveAnnouncementsResponse, CreateLeadRequest, CreateLeadResponse,
};

const MAX_EMAIL_BYTES: usize = 320;
const MAX_ACQUISITION_BYTES: usize = 200;

impl CopepodClient {
    /// Add a visitor to an audience list (double opt-in) via
    /// `POST /api/platform/orgs/{org}/apps/{app}/leads`.
    ///
    /// Requires an API key with scope `leads:write`; no Bearer token is sent.
    /// Arguments are validated before any request and rejected with
    /// [`CopepodError::InvalidArgument`]. Every well-formed request answers
    /// `accepted: true` whether the address is new, pending, confirmed or
    /// unsubscribed, so show the same message in every case.
    ///
    /// Server errors arrive as [`CopepodError::Api`]; branch on its `code`,
    /// not the message: 400 `invalid_lead`, 422 `turnstile_failed`, 429
    /// `rate_limited`, 503 `turnstile_unavailable` or `audience_upgrading`.
    /// 401, 403 (missing scope, wrong or inactive app), 404 and 503 (the
    /// server has no `COPEPOD_EXTERNAL_URL`) carry no `code`. See
    /// `docs/audience.md`.
    pub async fn create_lead(
        &self,
        org_id: &str,
        app_id: &str,
        lead: &CreateLeadRequest,
    ) -> Result<CreateLeadResponse> {
        validate_app_scope(org_id, app_id)?;
        let lead = validate_lead(lead)?;
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/leads");
        let response = self
            .api_key_request(Method::POST, &path)?
            .json(&lead)
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

fn invalid(message: impl Into<String>) -> CopepodError {
    CopepodError::InvalidArgument(message.into())
}

/// An acquisition field as the server reads it: trimmed, at most 200 bytes, no
/// control characters. Returns the trimmed value.
fn check_acquisition_field<'a>(name: &str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    if value.len() > MAX_ACQUISITION_BYTES || value.chars().any(char::is_control) {
        return Err(invalid(format!(
            "acquisition {name} must be at most 200 bytes without control characters"
        )));
    }
    Ok(value)
}

/// Mirror the server's `invalid_lead` rules, trimming first, and return the
/// request to send (country normalised).
fn validate_lead(lead: &CreateLeadRequest) -> Result<CreateLeadRequest> {
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
    validate_consent_text(&lead.consent_text)?;
    let mut out = lead.clone();
    if let Some(country) = &lead.country {
        let country = country.trim().to_ascii_uppercase();
        if country.len() != 2 || !country.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(invalid("country must be an ISO 3166-1 alpha-2 code"));
        }
        out.country = Some(country);
    }
    if let Some(ip) = &lead.client_ip {
        if ip.trim().parse::<IpAddr>().is_err() {
            return Err(invalid(
                "client_ip must be a single IP address; if you have an X-Forwarded-For list, pass its first hop",
            ));
        }
    }
    if let Some(a) = &lead.acquisition {
        for (name, value) in [
            ("utm_source", &a.utm_source),
            ("utm_medium", &a.utm_medium),
            ("utm_campaign", &a.utm_campaign),
            ("ref", &a.referral),
            ("referrer_host", &a.referrer_host),
        ] {
            if let Some(value) = value {
                check_acquisition_field(name, value)?;
            }
        }
        if let Some(path) = &a.landing_path {
            let path = check_acquisition_field("landing_path", path)?;
            if !path.is_empty() && !path.starts_with('/') {
                return Err(invalid("acquisition landing_path must start with /"));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::validate_lead;
    use crate::models::{CreateLeadRequest, LeadAcquisition};

    fn lead() -> CreateLeadRequest {
        CreateLeadRequest {
            list_key: "newsletter".into(),
            email: "a@b.co".into(),
            consent_text: "ok".into(),
            ..Default::default()
        }
    }

    fn acq(f: impl FnOnce(&mut LeadAcquisition)) -> CreateLeadRequest {
        let mut a = LeadAcquisition::default();
        f(&mut a);
        CreateLeadRequest {
            acquisition: Some(a),
            ..lead()
        }
    }

    #[test]
    fn accepts_valid_leads_and_normalises_country() {
        assert!(validate_lead(&lead()).is_ok());
        for (input, want) in [("gb", "GB"), (" GB ", "GB"), ("de", "DE")] {
            let r = CreateLeadRequest {
                country: Some(input.into()),
                ..lead()
            };
            assert_eq!(validate_lead(&r).unwrap().country.as_deref(), Some(want));
        }
        let ok = CreateLeadRequest {
            client_ip: Some(" 2001:db8::1 ".into()),
            ..lead()
        };
        assert!(validate_lead(&ok).is_ok());
        assert!(validate_lead(&acq(|a| a.landing_path = Some(" /p?x=1".into()))).is_ok());
    }

    #[test]
    fn rejects_what_the_server_rejects() {
        let mut bad = vec![
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
                consent_text: "a\0b".into(),
                ..lead()
            },
            CreateLeadRequest {
                country: Some("GBR".into()),
                ..lead()
            },
            CreateLeadRequest {
                country: Some("G1".into()),
                ..lead()
            },
            CreateLeadRequest {
                client_ip: Some("not-an-ip".into()),
                ..lead()
            },
            CreateLeadRequest {
                client_ip: Some("203.0.113.9, 10.0.0.1".into()),
                ..lead()
            },
            acq(|a| a.utm_source = Some("x".repeat(201))),
            acq(|a| a.referral = Some("a\nb".into())),
            acq(|a| a.landing_path = Some("pricing".into())),
        ];
        bad.push(CreateLeadRequest {
            client_ip: Some("".into()),
            ..lead()
        });
        for request in &bad {
            assert!(validate_lead(request).is_err(), "{request:?}");
        }
    }

    #[test]
    fn xff_list_error_tells_the_caller_to_pass_the_first_hop() {
        let r = CreateLeadRequest {
            client_ip: Some("1.2.3.4, 5.6.7.8".into()),
            ..lead()
        };
        let msg = validate_lead(&r).unwrap_err().to_string();
        assert!(msg.contains("first hop"), "{msg}");
    }
}
