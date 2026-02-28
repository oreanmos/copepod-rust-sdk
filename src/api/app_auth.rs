use crate::auth::TokenPair;
use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::auth::AuthResponse;

impl CopepodClient {
    /// Log in as an app user.
    pub async fn app_login(
        &self,
        org_id: &str,
        app_id: &str,
        identity: &str,
        password: &str,
    ) -> Result<AuthResponse> {
        let path = format!("api/orgs/{}/apps/{}/auth/login", org_id, app_id);
        let body = serde_json::json!({ "identity": identity, "password": password });
        let resp: AuthResponse = self.post(&path, &body).await?;
        self.token_store
            .set(TokenPair {
                token: resp.token.clone(),
                refresh_token: resp.refresh_token.clone(),
                expires_at: None,
            })
            .await;
        Ok(resp)
    }

    /// Register a new app user.
    pub async fn app_register(
        &self,
        org_id: &str,
        app_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<AuthResponse> {
        let path = format!("api/orgs/{}/apps/{}/auth/register", org_id, app_id);
        self.post(&path, body).await
    }

    /// Refresh the app user token.
    pub async fn app_refresh(
        &self,
        org_id: &str,
        app_id: &str,
    ) -> Result<AuthResponse> {
        let pair = self
            .token_store
            .get()
            .await
            .ok_or_else(|| crate::error::CopepodError::Auth("No token to refresh".into()))?;

        let path = format!("api/orgs/{}/apps/{}/auth/refresh", org_id, app_id);
        let body = serde_json::json!({ "refresh_token": pair.refresh_token });
        let resp: AuthResponse = self.post(&path, &body).await?;
        self.token_store
            .set(TokenPair {
                token: resp.token.clone(),
                refresh_token: resp.refresh_token.clone(),
                expires_at: None,
            })
            .await;
        Ok(resp)
    }

    /// Request email verification for an app user.
    pub async fn request_verification(
        &self,
        org_id: &str,
        app_id: &str,
        email: &str,
    ) -> Result<()> {
        let path = format!("api/orgs/{}/apps/{}/auth/request-verification", org_id, app_id);
        let body = serde_json::json!({ "email": email });
        self.post_empty(&path, &body).await
    }

    /// Confirm email verification for an app user.
    pub async fn confirm_verification(
        &self,
        org_id: &str,
        app_id: &str,
        token: &str,
    ) -> Result<()> {
        let path = format!("api/orgs/{}/apps/{}/auth/confirm-verification", org_id, app_id);
        let body = serde_json::json!({ "token": token });
        self.post_empty(&path, &body).await
    }

    /// Request a password reset for an app user.
    pub async fn request_password_reset(
        &self,
        org_id: &str,
        app_id: &str,
        email: &str,
    ) -> Result<()> {
        let path = format!("api/orgs/{}/apps/{}/auth/request-password-reset", org_id, app_id);
        let body = serde_json::json!({ "email": email });
        self.post_empty(&path, &body).await
    }

    /// Confirm a password reset for an app user.
    pub async fn confirm_password_reset(
        &self,
        org_id: &str,
        app_id: &str,
        token: &str,
        password: &str,
    ) -> Result<()> {
        let path = format!("api/orgs/{}/apps/{}/auth/confirm-password-reset", org_id, app_id);
        let body = serde_json::json!({ "token": token, "password": password });
        self.post_empty(&path, &body).await
    }
}
