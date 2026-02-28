use crate::client::CopepodClient;
use crate::error::Result;

impl CopepodClient {
    /// Get the OAuth authorization URL for a provider.
    pub async fn get_oauth_authorize_url(
        &self,
        provider: &str,
    ) -> Result<serde_json::Value> {
        self.get(&format!(
            "api/platform/auth/oauth/{}/authorize",
            provider
        ))
        .await
    }

    /// Handle OAuth callback (exchange code for tokens).
    pub async fn oauth_callback(
        &self,
        provider: &str,
        code: &str,
        state: &str,
    ) -> Result<crate::models::AuthResponse> {
        let url = format!(
            "api/platform/auth/oauth/{}/callback?code={}&state={}",
            provider, code, state
        );
        self.get(&url).await
    }
}
