use bytes::Bytes;
use reqwest::header::AUTHORIZATION;
use reqwest::Method;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{ListResult, Ticket, TicketComment, TicketAttachment};

impl CopepodClient {
    // -- App-scoped ticket endpoints --

    /// Create a ticket in an app.
    pub async fn create_ticket(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Ticket> {
        let path = format!("api/orgs/{}/apps/{}/tickets", org_id, app_id);
        self.post(&path, body).await
    }

    /// List tickets in an app.
    pub async fn list_app_tickets(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<ListResult<Ticket>> {
        let path = format!("api/orgs/{}/apps/{}/tickets", org_id, app_id);
        self.get(&path).await
    }

    /// Get a specific ticket in an app.
    pub async fn get_app_ticket(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
    ) -> Result<Ticket> {
        let path = format!("api/orgs/{}/apps/{}/tickets/{}", org_id, app_id, ticket_id);
        self.get(&path).await
    }

    /// Close a ticket in an app.
    pub async fn close_ticket(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
    ) -> Result<Ticket> {
        let path = format!(
            "api/orgs/{}/apps/{}/tickets/{}/close",
            org_id, app_id, ticket_id
        );
        self.post(&path, &serde_json::json!({})).await
    }

    // -- Ticket comments --

    /// Add a comment to a ticket.
    pub async fn add_comment(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<TicketComment> {
        let path = format!(
            "api/orgs/{}/apps/{}/tickets/{}/comments",
            org_id, app_id, ticket_id
        );
        self.post(&path, body).await
    }

    /// List comments on a ticket.
    pub async fn list_comments(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
    ) -> Result<ListResult<TicketComment>> {
        let path = format!(
            "api/orgs/{}/apps/{}/tickets/{}/comments",
            org_id, app_id, ticket_id
        );
        self.get(&path).await
    }

    // -- Ticket attachments --

    /// Upload an attachment to a ticket.
    pub async fn upload_attachment(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
        data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<TicketAttachment> {
        let path = format!(
            "api/orgs/{}/apps/{}/tickets/{}/attachments",
            org_id, app_id, ticket_id
        );

        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str(content_type)
            .map_err(CopepodError::Http)?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let mut builder = self.request(Method::POST, &path).multipart(form);
        if let Some(pair) = self.token_store.get().await {
            builder = builder.header(AUTHORIZATION, format!("Bearer {}", pair.token));
        }

        let resp = builder.send().await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Download a ticket attachment.
    pub async fn download_attachment(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
        attachment_id: &str,
    ) -> Result<Bytes> {
        let path = format!(
            "api/orgs/{}/apps/{}/tickets/{}/attachments/{}",
            org_id, app_id, ticket_id, attachment_id
        );
        let resp = self
            .auth_request(Method::GET, &path)
            .await?
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.bytes().await?)
        } else {
            let status = resp.status();
            let body: Value = resp.json().await.unwrap_or_default();
            Err(CopepodError::Api {
                status: status.as_u16(),
                code: body.get("code").and_then(|v| v.as_str()).map(String::from),
                message: body
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Download failed")
                    .to_string(),
            })
        }
    }

    // -- Admin ticket endpoints --

    /// List all tickets (admin).
    pub async fn list_tickets(&self) -> Result<ListResult<Ticket>> {
        self.get("api/admin/tickets").await
    }

    /// Update a ticket (admin).
    pub async fn update_ticket(
        &self,
        ticket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Ticket> {
        self.patch(&format!("api/admin/tickets/{}", ticket_id), body)
            .await
    }

    /// Get ticket statistics (admin).
    pub async fn get_ticket_stats(&self) -> Result<Value> {
        self.get("api/admin/tickets/stats").await
    }
}
