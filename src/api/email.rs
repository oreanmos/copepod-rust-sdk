use reqwest::Method;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    EmailDeliveryEnqueueResponse, EmailDeliveryStatusBatchRequest,
    EmailDeliveryStatusBatchResponse, EmailDeliveryStatusResponse, TransactionalEmailRequest,
    MAX_EMAIL_DELIVERY_STATUS_BATCH,
};

const MAX_RESOURCE_ID_BYTES: usize = 128;
const MAX_RECIPIENT_BYTES: usize = 320;
const MAX_SUBJECT_BYTES: usize = 255;
const MAX_BODY_HTML_BYTES: usize = 256 * 1024;

impl CopepodClient {
    /// Durably enqueue a transactional email for an application.
    ///
    /// `idempotency_key` must contain 1–128 visible ASCII characters. Reusing
    /// the key with the same request returns the original delivery; reusing it
    /// with different content is rejected by the server.
    ///
    /// This operation uses the API key configured with
    /// [`CopepodClient::builder`], not a Bearer token.
    pub async fn enqueue_transactional_email(
        &self,
        org_id: &str,
        app_id: &str,
        idempotency_key: &str,
        request: &TransactionalEmailRequest,
    ) -> Result<EmailDeliveryEnqueueResponse> {
        validate_app_scope(org_id, app_id)?;
        validate_idempotency_key(idempotency_key)?;
        validate_email_request(request)?;
        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/email/send");
        let request = self
            .api_key_request(Method::POST, &path)?
            .header("Idempotency-Key", idempotency_key)
            .json(request);
        let response = request.send().await?;

        Self::handle_response_pub(response).await
    }

    /// Fetch one transactional email delivery state.
    pub async fn get_email_delivery(
        &self,
        org_id: &str,
        app_id: &str,
        delivery_id: &str,
    ) -> Result<EmailDeliveryStatusResponse> {
        validate_app_scope(org_id, app_id)?;
        validate_resource_id("delivery ID", delivery_id)?;
        let path = delivery_path(org_id, app_id, delivery_id);
        let request = self.api_key_request(Method::GET, &path)?;
        let response = request.send().await?;
        Self::handle_response_pub(response).await
    }

    /// Fetch between 1 and 100 delivery states in one request.
    ///
    /// The server returns found deliveries in the same order as `delivery_ids`.
    pub async fn get_email_delivery_statuses(
        &self,
        org_id: &str,
        app_id: &str,
        delivery_ids: &[String],
    ) -> Result<EmailDeliveryStatusBatchResponse> {
        validate_app_scope(org_id, app_id)?;
        if delivery_ids.is_empty() || delivery_ids.len() > MAX_EMAIL_DELIVERY_STATUS_BATCH {
            return Err(CopepodError::InvalidArgument(format!(
                "between 1 and {MAX_EMAIL_DELIVERY_STATUS_BATCH} email delivery IDs are required"
            )));
        }
        for delivery_id in delivery_ids {
            validate_resource_id("delivery ID", delivery_id)?;
        }

        let path = format!("api/platform/orgs/{org_id}/apps/{app_id}/email/deliveries/status");
        let request = EmailDeliveryStatusBatchRequest {
            delivery_ids: delivery_ids.to_vec(),
        };
        let request = self.api_key_request(Method::POST, &path)?.json(&request);
        let response = request.send().await?;

        Self::handle_response_pub(response).await
    }

    /// Retry a terminally failed transactional email delivery.
    pub async fn retry_email_delivery(
        &self,
        org_id: &str,
        app_id: &str,
        delivery_id: &str,
    ) -> Result<EmailDeliveryStatusResponse> {
        self.email_delivery_action(org_id, app_id, delivery_id, "retry")
            .await
    }

    /// Cancel a pending or failed transactional email delivery.
    pub async fn cancel_email_delivery(
        &self,
        org_id: &str,
        app_id: &str,
        delivery_id: &str,
    ) -> Result<EmailDeliveryStatusResponse> {
        self.email_delivery_action(org_id, app_id, delivery_id, "cancel")
            .await
    }

    async fn email_delivery_action(
        &self,
        org_id: &str,
        app_id: &str,
        delivery_id: &str,
        action: &str,
    ) -> Result<EmailDeliveryStatusResponse> {
        validate_app_scope(org_id, app_id)?;
        validate_resource_id("delivery ID", delivery_id)?;
        let path = format!("{}/{action}", delivery_path(org_id, app_id, delivery_id));
        let request = self.api_key_request(Method::POST, &path)?;
        let response = request.send().await?;
        Self::handle_response_pub(response).await
    }
}

fn delivery_path(org_id: &str, app_id: &str, delivery_id: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/email/deliveries/{delivery_id}")
}

fn validate_idempotency_key(key: &str) -> Result<()> {
    let valid_length = (1..=128).contains(&key.len());
    let visible_ascii = key.bytes().all(|byte| (0x21..=0x7e).contains(&byte));
    if valid_length && visible_ascii {
        return Ok(());
    }

    Err(CopepodError::InvalidArgument(
        "idempotency key must contain 1-128 visible ASCII characters".into(),
    ))
}

fn validate_app_scope(org_id: &str, app_id: &str) -> Result<()> {
    validate_resource_id("organization ID", org_id)?;
    validate_resource_id("application ID", app_id)
}

fn validate_resource_id(label: &str, value: &str) -> Result<()> {
    let valid_length = (1..=MAX_RESOURCE_ID_BYTES).contains(&value.len());
    let path_safe = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid_length && path_safe {
        return Ok(());
    }

    Err(CopepodError::InvalidArgument(format!(
        "{label} must contain 1-{MAX_RESOURCE_ID_BYTES} ASCII letters, digits, hyphens, or underscores"
    )))
}

fn validate_email_request(request: &TransactionalEmailRequest) -> Result<()> {
    let recipient = request.to.trim();
    if recipient.is_empty()
        || recipient.len() > MAX_RECIPIENT_BYTES
        || !looks_like_email_address(recipient)
    {
        return Err(CopepodError::InvalidArgument(
            "email recipient must be a valid address of at most 320 bytes".into(),
        ));
    }

    let subject = request.subject.trim();
    if subject.is_empty()
        || subject.len() > MAX_SUBJECT_BYTES
        || subject.contains(['\r', '\n', '\0'])
    {
        return Err(CopepodError::InvalidArgument(
            "email subject must contain 1-255 bytes without CR, LF, or null characters".into(),
        ));
    }

    if request.body_html.trim().is_empty()
        || request.body_html.len() > MAX_BODY_HTML_BYTES
        || request.body_html.contains('\0')
    {
        return Err(CopepodError::InvalidArgument(format!(
            "email HTML body must contain 1-{MAX_BODY_HTML_BYTES} bytes without null characters"
        )));
    }

    Ok(())
}

fn looks_like_email_address(value: &str) -> bool {
    let mut parts = value.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    !local.is_empty()
        && !domain.is_empty()
        && parts.next().is_none()
        && !value.chars().any(char::is_whitespace)
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.contains('.')
}

#[cfg(test)]
mod tests {
    use super::{validate_email_request, validate_idempotency_key, validate_resource_id};
    use crate::{CopepodError, TransactionalEmailRequest};

    #[test]
    fn validates_idempotency_key_wire_constraints() {
        assert!(validate_idempotency_key("invite:participant_1:v2").is_ok());
        assert!(matches!(
            validate_idempotency_key(""),
            Err(CopepodError::InvalidArgument(_))
        ));
        assert!(validate_idempotency_key(&"x".repeat(129)).is_err());
        assert!(validate_idempotency_key("contains space").is_err());
        assert!(validate_idempotency_key("contains\nnewline").is_err());
    }

    #[test]
    fn rejects_resource_ids_that_can_escape_a_path_segment() {
        assert!(validate_resource_id("delivery ID", "01JTEST_delivery-1").is_ok());
        assert!(validate_resource_id("delivery ID", "../cancel").is_err());
        assert!(validate_resource_id("delivery ID", "has/slash").is_err());
    }

    #[test]
    fn validates_email_payload_before_sending() {
        let valid = TransactionalEmailRequest {
            to: "traveler@example.com".into(),
            subject: "Trip invitation".into(),
            body_html: "<p>Join the trip.</p>".into(),
        };
        assert!(validate_email_request(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.subject = "Invite\r\nBcc: victim@example.com".into();
        assert!(validate_email_request(&invalid).is_err());

        invalid = valid.clone();
        invalid.to = "not-an-address".into();
        assert!(validate_email_request(&invalid).is_err());

        invalid = valid;
        invalid.body_html = " \n ".into();
        assert!(validate_email_request(&invalid).is_err());
    }
}
