use serde_json::Value;

use crate::auth::TokenPair;
use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::auth::AuthResponse;

impl CopepodClient {
    /// Log in with email and password. Stores tokens automatically.
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse> {
        let body = serde_json::json!({ "email": email, "password": password });
        let resp: AuthResponse = self.post("api/auth/login", &body).await?;
        self.token_store
            .set(TokenPair {
                token: resp.token.clone(),
                refresh_token: resp.refresh_token.clone(),
                expires_at: None,
            })
            .await;
        Ok(resp)
    }

    /// Refresh the current access token using the stored refresh token.
    pub async fn refresh(&self) -> Result<AuthResponse> {
        let pair = self
            .token_store
            .get()
            .await
            .ok_or_else(|| crate::error::CopepodError::Auth("No token to refresh".into()))?;

        let body = serde_json::json!({ "refresh_token": pair.refresh_token });
        let resp: AuthResponse = self.post("api/auth/refresh", &body).await?;
        self.token_store
            .set(TokenPair {
                token: resp.token.clone(),
                refresh_token: resp.refresh_token.clone(),
                expires_at: None,
            })
            .await;
        Ok(resp)
    }

    /// Log out and clear the token store.
    pub async fn logout(&self) -> Result<()> {
        let _ = self.post_empty("api/auth/logout", &serde_json::json!({})).await;
        self.token_store.clear().await;
        Ok(())
    }

    /// Verify an MFA code during login.
    pub async fn mfa_verify(&self, mfa_token: &str, code: &str) -> Result<AuthResponse> {
        let body = serde_json::json!({ "mfa_token": mfa_token, "code": code });
        let resp: AuthResponse = self.post("api/auth/mfa/verify", &body).await?;
        self.token_store
            .set(TokenPair {
                token: resp.token.clone(),
                refresh_token: resp.refresh_token.clone(),
                expires_at: None,
            })
            .await;
        Ok(resp)
    }

    /// Start MFA setup (returns provisioning URI, secret, etc.).
    pub async fn mfa_setup(&self) -> Result<Value> {
        self.get("api/auth/mfa/setup").await
    }

    /// Enable MFA with a TOTP code.
    pub async fn mfa_enable(&self, code: &str) -> Result<()> {
        let body = serde_json::json!({ "code": code });
        self.post_empty("api/auth/mfa/enable", &body).await
    }

    /// Disable MFA with a TOTP code.
    pub async fn mfa_disable(&self, code: &str) -> Result<()> {
        let body = serde_json::json!({ "code": code });
        self.post_empty("api/auth/mfa/disable", &body).await
    }
}
