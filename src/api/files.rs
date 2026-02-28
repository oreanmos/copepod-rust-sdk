use bytes::Bytes;
use reqwest::header::AUTHORIZATION;
use reqwest::Method;
use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};

impl CopepodClient {
    /// Upload a file to a record field.
    #[allow(clippy::too_many_arguments)]
    pub async fn upload_file(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        record_id: &str,
        field: &str,
        data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<Value> {
        let path = format!(
            "api/orgs/{}/apps/{}/collections/{}/records/{}/files/{}",
            org_id, app_id, collection, record_id, field
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

    /// Download a file from a record.
    pub async fn download_file(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        record_id: &str,
        filename: &str,
    ) -> Result<Bytes> {
        let path = format!(
            "api/orgs/{}/apps/{}/collections/{}/records/{}/files/{}",
            org_id, app_id, collection, record_id, filename
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

    /// Delete a file from a record.
    pub async fn delete_file(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        record_id: &str,
        filename: &str,
    ) -> Result<()> {
        let path = format!(
            "api/orgs/{}/apps/{}/collections/{}/records/{}/files/{}",
            org_id, app_id, collection, record_id, filename
        );
        self.delete(&path).await
    }
}
