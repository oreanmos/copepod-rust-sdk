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
    pub async fn get_signed_file(&self, app_id: &str, key: &str) -> Result<bytes::Bytes> {
        let resp = self
            .auth_request(
                reqwest::Method::GET,
                &format!("api/platform/apps/{}/files/signed/{}", app_id, key),
            )
            .await?
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(resp.bytes().await?)
        } else {
            let bytes = resp.bytes().await.unwrap_or_default();
            let (code, message) = crate::client::decode_error_body(&bytes);
            Err(crate::error::CopepodError::Api {
                status: status.as_u16(),
                code,
                message,
            })
        }
    }
}
