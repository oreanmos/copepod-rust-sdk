use bytes::Bytes;
use reqwest::header::AUTHORIZATION;
use reqwest::Method;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    ItemsResponse, ListResult, Ticket, TicketAttachment, TicketComment, TicketListQuery,
    TicketStats,
};

impl CopepodClient {
    async fn get_ticket_list_with_query(
        &self,
        path: &str,
        query: &TicketListQuery,
    ) -> Result<ListResult<Ticket>> {
        let mut params = Vec::<(&str, String)>::new();
        if let Some(value) = query.page {
            params.push(("page", value.to_string()));
        }
        if let Some(value) = query.per_page {
            params.push(("per_page", value.to_string()));
        }
        if let Some(value) = query.app_id.as_ref() {
            params.push(("app_id", value.clone()));
        }
        if let Some(value) = query.org_id.as_ref() {
            params.push(("org_id", value.clone()));
        }
        if let Some(value) = query.user_id.as_ref() {
            params.push(("user_id", value.clone()));
        }
        if let Some(value) = query.status.as_ref() {
            params.push(("status", value.clone()));
        }
        if let Some(value) = query.priority.as_ref() {
            params.push(("priority", value.clone()));
        }
        if let Some(value) = query.category.as_ref() {
            params.push(("category", value.clone()));
        }
        if let Some(value) = query.search.as_ref() {
            params.push(("search", value.clone()));
        }
        if let Some(value) = query.assigned_to_id.as_ref() {
            params.push(("assigned_to_id", value.clone()));
        }
        if let Some(value) = query.start_date.as_ref() {
            params.push(("start_date", value.clone()));
        }
        if let Some(value) = query.end_date.as_ref() {
            params.push(("end_date", value.clone()));
        }
        if let Some(value) = query.sort.as_ref() {
            params.push(("sort", value.clone()));
        }
        let resp = self
            .auth_request(Method::GET, path)
            .await?
            .query(&params)
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }

    // -- App-scoped ticket endpoints --

    /// Create a ticket in an app.
    pub async fn create_ticket(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Ticket> {
        let path = format!("api/platform/orgs/{}/apps/{}/tickets", org_id, app_id);
        self.post(&path, body).await
    }

    /// List tickets in an app.
    pub async fn list_app_tickets(&self, org_id: &str, app_id: &str) -> Result<ListResult<Ticket>> {
        let path = format!("api/platform/orgs/{}/apps/{}/tickets", org_id, app_id);
        self.get(&path).await
    }

    /// List tickets in an app with filters.
    pub async fn list_app_tickets_filtered(
        &self,
        org_id: &str,
        app_id: &str,
        query: &TicketListQuery,
    ) -> Result<ListResult<Ticket>> {
        let path = format!("api/platform/orgs/{}/apps/{}/tickets", org_id, app_id);
        self.get_ticket_list_with_query(&path, query).await
    }

    /// Get a specific ticket in an app.
    pub async fn get_app_ticket(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
    ) -> Result<Ticket> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/tickets/{}",
            org_id, app_id, ticket_id
        );
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
            "api/platform/orgs/{}/apps/{}/tickets/{}/close",
            org_id, app_id, ticket_id
        );
        self.post(&path, &serde_json::json!({})).await
    }

    /// Reopen a ticket in an app.
    pub async fn reopen_ticket(
        &self,
        org_id: &str,
        app_id: &str,
        ticket_id: &str,
    ) -> Result<Ticket> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/tickets/{}/reopen",
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
            "api/platform/orgs/{}/apps/{}/tickets/{}/comments",
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
    ) -> Result<ItemsResponse<TicketComment>> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/tickets/{}/comments",
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
            "api/platform/orgs/{}/apps/{}/tickets/{}/attachments",
            org_id, app_id, ticket_id
        );

        let mut builder = self
            .request(Method::POST, &path)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .header(
                reqwest::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            )
            .body(data);
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
            "api/platform/orgs/{}/apps/{}/tickets/{}/attachments/{}",
            org_id, app_id, ticket_id, attachment_id
        );
        let resp = self.auth_request(Method::GET, &path).await?.send().await?;

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
        self.get("api/platform/tickets").await
    }

    /// List all tickets (admin) with filters.
    pub async fn list_tickets_filtered(
        &self,
        query: &TicketListQuery,
    ) -> Result<ListResult<Ticket>> {
        self.get_ticket_list_with_query("api/platform/tickets", query)
            .await
    }

    /// Get a ticket (admin).
    pub async fn get_ticket(&self, ticket_id: &str) -> Result<Ticket> {
        self.get(&format!("api/platform/tickets/{}", ticket_id))
            .await
    }

    /// Update a ticket (admin).
    pub async fn update_ticket(
        &self,
        ticket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Ticket> {
        self.patch(&format!("api/platform/tickets/{}", ticket_id), body)
            .await
    }

    /// Get ticket statistics (admin).
    pub async fn get_ticket_stats(&self) -> Result<TicketStats> {
        self.get("api/platform/tickets/stats").await
    }

    /// List comments on a ticket (admin).
    pub async fn list_ticket_comments_admin(
        &self,
        ticket_id: &str,
    ) -> Result<ItemsResponse<TicketComment>> {
        self.get(&format!("api/platform/tickets/{}/comments", ticket_id))
            .await
    }

    /// Add a comment to a ticket (admin).
    pub async fn add_ticket_comment_admin(
        &self,
        ticket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<TicketComment> {
        self.post(
            &format!("api/platform/tickets/{}/comments", ticket_id),
            body,
        )
        .await
    }

    // -- Org-scoped support ticket endpoints --

    /// List support tickets for an organization.
    pub async fn list_support_tickets(&self, org_id: &str) -> Result<ListResult<Ticket>> {
        self.get(&format!("api/platform/orgs/{}/support/tickets", org_id))
            .await
    }

    /// Create a support ticket for an organization.
    pub async fn create_support_ticket(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Ticket> {
        self.post(
            &format!("api/platform/orgs/{}/support/tickets", org_id),
            body,
        )
        .await
    }

    /// Get a support ticket by ID.
    pub async fn get_support_ticket(&self, org_id: &str, ticket_id: &str) -> Result<Ticket> {
        self.get(&format!(
            "api/platform/orgs/{}/support/tickets/{}",
            org_id, ticket_id
        ))
        .await
    }

    /// Update a support ticket.
    pub async fn update_support_ticket(
        &self,
        org_id: &str,
        ticket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Ticket> {
        self.patch(
            &format!("api/platform/orgs/{}/support/tickets/{}", org_id, ticket_id),
            body,
        )
        .await
    }

    /// Add a comment to a support ticket.
    pub async fn add_support_comment(
        &self,
        org_id: &str,
        ticket_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<TicketComment> {
        self.post(
            &format!(
                "api/platform/orgs/{}/support/tickets/{}/comment",
                org_id, ticket_id
            ),
            body,
        )
        .await
    }

    /// Assign a support ticket to a user.
    pub async fn assign_support_ticket(
        &self,
        org_id: &str,
        ticket_id: &str,
        user_id: &str,
    ) -> Result<Value> {
        let body = serde_json::json!({ "user_id": user_id });
        self.post(
            &format!(
                "api/platform/orgs/{}/support/tickets/{}/assign",
                org_id, ticket_id
            ),
            &body,
        )
        .await
    }

    /// Close a support ticket.
    pub async fn close_support_ticket(&self, org_id: &str, ticket_id: &str) -> Result<Value> {
        self.post(
            &format!(
                "api/platform/orgs/{}/support/tickets/{}/close",
                org_id, ticket_id
            ),
            &serde_json::json!({}),
        )
        .await
    }
}
