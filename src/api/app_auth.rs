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

    /// Request an email change for an app user.
    pub async fn request_email_change(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        new_email: &str,
    ) -> Result<()> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/request-email-change",
            org_id, app_id, collection
        );
        let body = serde_json::json!({ "new_email": new_email });
        self.post_empty(&path, &body).await
    }

    /// Confirm an email change for an app user.
    pub async fn confirm_email_change(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        token: &str,
    ) -> Result<()> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/confirm-email-change",
            org_id, app_id, collection
        );
        let body = serde_json::json!({ "token": token });
        self.post_empty(&path, &body).await
    }

    /// Enroll in MFA for an app user.
    pub async fn app_mfa_enroll(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<serde_json::Value> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/mfa/enroll",
            org_id, app_id, collection
        );
        self.post(&path, &serde_json::json!({})).await
    }

    /// Confirm MFA enrollment for an app user.
    pub async fn app_mfa_confirm_enroll(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        code: &str,
    ) -> Result<()> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/mfa/confirm-enroll",
            org_id, app_id, collection
        );
        let body = serde_json::json!({ "code": code });
        self.post_empty(&path, &body).await
    }

    /// Disable MFA for an app user.
    pub async fn app_mfa_disable(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        code: &str,
    ) -> Result<()> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/mfa/disable",
            org_id, app_id, collection
        );
        let body = serde_json::json!({ "code": code });
        self.post_empty(&path, &body).await
    }

    /// Verify MFA code during app user login.
    pub async fn app_mfa_verify(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        mfa_token: &str,
        code: &str,
    ) -> Result<AuthResponse> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/mfa/verify",
            org_id, app_id, collection
        );
        let body = serde_json::json!({ "mfa_token": mfa_token, "code": code });
        self.post(&path, &body).await
    }

    /// Use a recovery code for MFA during app user login.
    pub async fn app_mfa_recovery(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        mfa_token: &str,
        recovery_code: &str,
    ) -> Result<AuthResponse> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/mfa/recovery",
            org_id, app_id, collection
        );
        let body = serde_json::json!({ "mfa_token": mfa_token, "recovery_code": recovery_code });
        self.post(&path, &body).await
    }

    /// Set password for an app user (admin).
    pub async fn admin_set_password(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        user_id: &str,
        password: &str,
    ) -> Result<serde_json::Value> {
        let path = format!(
            "api/platform/orgs/{}/apps/{}/auth/{}/{}/set-password",
            org_id, app_id, collection, user_id
        );
        let body = serde_json::json!({ "password": password });
        self.post(&path, &body).await
    }
}
