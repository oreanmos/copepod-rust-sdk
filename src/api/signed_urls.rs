use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::SignedUrlResponse;

impl CopepodClient {
    /// Create a signed URL for a file.
    pub async fn create_signed_url(
        &self,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<SignedUrlResponse> {
        self.post(&format!("api/platform/apps/{}/files/sign", app_id), body)
            .await
    }

    /// Download a file using a signed key.
    pub async fn get_signed_file(
        &self,
        app_id: &str,
        key: &str,
    ) -> Result<bytes::Bytes> {
        let resp = self
            .auth_request(
                reqwest::Method::GET,
                &format!(
                    "api/platform/apps/{}/files/signed/{}",
                    app_id, key
                ),
            )
            .await?
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(resp.bytes().await?)
        } else {
            let body: serde_json::Value =
                resp.json().await.unwrap_or_default();
            Err(crate::error::CopepodError::Api {
                status: status.as_u16(),
                code: body
                    .get("code")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                message: body
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string(),
            })
        }
    }
}
