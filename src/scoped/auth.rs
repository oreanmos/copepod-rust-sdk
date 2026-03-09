use serde::Serialize;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{AppLoginResult, AuthResponse, MfaEnrollResponse};

/// App auth helpers bound to a specific auth collection.
#[derive(Debug, Clone)]
pub struct ScopedAppAuthClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
    app_id: String,
    collection: String,
}

impl<'a> ScopedAppAuthClient<'a> {
    pub(crate) fn new(
        client: &'a CopepodClient,
        org_id: &str,
        app_id: &str,
        collection: impl Into<String>,
    ) -> Self {
        Self {
            client,
            org_id: org_id.to_string(),
            app_id: app_id.to_string(),
            collection: collection.into(),
        }
    }

    /// Return the bound auth collection name.
    pub fn collection(&self) -> &str {
        &self.collection
    }

    /// Log in as an app user.
    pub async fn login(&self, identity: &str, password: &str) -> Result<AppLoginResult> {
        self.client
            .app_login(
                &self.org_id,
                &self.app_id,
                &self.collection,
                identity,
                password,
            )
            .await
    }

    /// Register a new app user.
    pub async fn register(&self, body: &impl Serialize) -> Result<AuthResponse> {
        self.client
            .app_register(&self.org_id, &self.app_id, &self.collection, body)
            .await
    }

    /// Refresh the current app user token.
    pub async fn refresh(&self) -> Result<AuthResponse> {
        self.client
            .app_refresh(&self.org_id, &self.app_id, &self.collection)
            .await
    }

    /// Request email verification for an app user.
    pub async fn request_verification(&self, email: &str) -> Result<()> {
        self.client
            .request_verification(&self.org_id, &self.app_id, &self.collection, email)
            .await
    }

    /// Confirm email verification for an app user.
    pub async fn confirm_verification(&self, token: &str) -> Result<()> {
        self.client
            .confirm_verification(&self.org_id, &self.app_id, &self.collection, token)
            .await
    }

    /// Request a password reset for an app user.
    pub async fn request_password_reset(&self, email: &str) -> Result<()> {
        self.client
            .request_password_reset(&self.org_id, &self.app_id, &self.collection, email)
            .await
    }

    /// Confirm a password reset for an app user.
    pub async fn confirm_password_reset(&self, token: &str, password: &str) -> Result<()> {
        self.client
            .confirm_password_reset(
                &self.org_id,
                &self.app_id,
                &self.collection,
                token,
                password,
            )
            .await
    }

    /// Request an email change for the current app user.
    pub async fn request_email_change(&self, new_email: &str) -> Result<()> {
        self.client
            .request_email_change(&self.org_id, &self.app_id, &self.collection, new_email)
            .await
    }

    /// Confirm an email change for the current app user.
    pub async fn confirm_email_change(&self, token: &str) -> Result<()> {
        self.client
            .confirm_email_change(&self.org_id, &self.app_id, &self.collection, token)
            .await
    }

    /// Start app MFA enrollment.
    pub async fn mfa_enroll(&self) -> Result<MfaEnrollResponse> {
        self.client
            .app_mfa_enroll(&self.org_id, &self.app_id, &self.collection)
            .await
    }

    /// Confirm app MFA enrollment.
    pub async fn mfa_confirm_enroll(&self, code: &str) -> Result<()> {
        self.client
            .app_mfa_confirm_enroll(&self.org_id, &self.app_id, &self.collection, code)
            .await
    }

    /// Disable app MFA.
    pub async fn mfa_disable(&self, code: &str) -> Result<()> {
        self.client
            .app_mfa_disable(&self.org_id, &self.app_id, &self.collection, code)
            .await
    }

    /// Verify an MFA challenge during login.
    pub async fn mfa_verify(&self, mfa_token: &str, code: &str) -> Result<AuthResponse> {
        self.client
            .app_mfa_verify(
                &self.org_id,
                &self.app_id,
                &self.collection,
                mfa_token,
                code,
            )
            .await
    }

    /// Use a recovery code during MFA login.
    pub async fn mfa_recovery(&self, mfa_token: &str, recovery_code: &str) -> Result<AuthResponse> {
        self.client
            .app_mfa_recovery(
                &self.org_id,
                &self.app_id,
                &self.collection,
                mfa_token,
                recovery_code,
            )
            .await
    }
}
