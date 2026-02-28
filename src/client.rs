use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crate::auth::{TokenPair, TokenStore};
use crate::error::{CopepodError, Result};

/// The main client for interacting with the Copepod API.
#[derive(Debug, Clone)]
pub struct CopepodClient {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) token_store: Arc<TokenStore>,
    pub(crate) auto_refresh: bool,
}

/// Builder for constructing a [`CopepodClient`].
pub struct CopepodClientBuilder {
    base_url: Option<String>,
    token: Option<String>,
    refresh_token: Option<String>,
    auto_refresh: bool,
}

impl CopepodClientBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            token: None,
            refresh_token: None,
            auto_refresh: true,
        }
    }

    /// Set the base URL of the Copepod server.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set an existing access token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set an existing refresh token.
    pub fn refresh_token(mut self, token: impl Into<String>) -> Self {
        self.refresh_token = Some(token.into());
        self
    }

    /// Enable or disable automatic token refresh (default: true).
    pub fn auto_refresh(mut self, enabled: bool) -> Self {
        self.auto_refresh = enabled;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<CopepodClient> {
        let base_url_str = self
            .base_url
            .ok_or_else(|| CopepodError::Auth("base_url is required".into()))?;

        let mut base_url = Url::parse(&base_url_str)?;
        // Ensure trailing slash for proper path joining
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }

        let token_store = if let Some(token) = self.token {
            Arc::new(TokenStore::with_token(TokenPair {
                token,
                refresh_token: self.refresh_token.unwrap_or_default(),
                expires_at: None,
            }))
        } else {
            Arc::new(TokenStore::new())
        };

        let http = reqwest::Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert("Accept", HeaderValue::from_static("application/json"));
                headers
            })
            .build()
            .map_err(CopepodError::Http)?;

        Ok(CopepodClient {
            http,
            base_url,
            token_store,
            auto_refresh: self.auto_refresh,
        })
    }
}

impl Default for CopepodClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CopepodClient {
    /// Create a new builder.
    pub fn builder() -> CopepodClientBuilder {
        CopepodClientBuilder::new()
    }

    /// Get a reference to the token store.
    pub fn token_store(&self) -> &Arc<TokenStore> {
        &self.token_store
    }

    /// Build a request with the given method and path (relative to base_url).
    pub(crate) fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self.base_url.join(path).unwrap_or_else(|_| {
            let mut u = self.base_url.clone();
            u.set_path(path);
            u
        });
        self.http.request(method, url)
    }

    /// Ensure we have a valid auth token, refreshing if needed.
    pub(crate) async fn ensure_auth(&self) -> Result<()> {
        if self.auto_refresh && self.token_store.needs_refresh().await {
            let pair = self
                .token_store
                .get()
                .await
                .ok_or_else(|| CopepodError::Auth("No token available for refresh".into()))?;

            if pair.refresh_token.is_empty() {
                return Err(CopepodError::Auth("No refresh token available".into()));
            }

            // Call the refresh endpoint
            let url = self.base_url.join("api/auth/refresh")?;
            let resp = self
                .http
                .post(url)
                .json(&serde_json::json!({ "refresh_token": pair.refresh_token }))
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                return Err(CopepodError::Api {
                    status,
                    code: body.get("code").and_then(|v| v.as_str()).map(String::from),
                    message: body
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Token refresh failed")
                        .to_string(),
                });
            }

            let auth_resp: crate::models::AuthResponse = resp.json().await?;
            self.token_store
                .set(TokenPair {
                    token: auth_resp.token.clone(),
                    refresh_token: auth_resp.refresh_token.clone(),
                    expires_at: None,
                })
                .await;
        }
        Ok(())
    }

    /// Add authorization header to a request builder.
    pub(crate) async fn auth_request(
        &self,
        method: Method,
        path: &str,
    ) -> Result<RequestBuilder> {
        self.ensure_auth().await?;
        let mut builder = self.request(method, path);
        if let Some(pair) = self.token_store.get().await {
            builder = builder.header(AUTHORIZATION, format!("Bearer {}", pair.token));
        }
        Ok(builder)
    }

    /// Perform an authenticated GET request and deserialize the response.
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self.auth_request(Method::GET, path).await?.send().await?;
        Self::handle_response(resp).await
    }

    /// Perform an authenticated POST request with a JSON body.
    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .auth_request(Method::POST, path)
            .await?
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// Perform an authenticated POST request with no response body.
    pub(crate) async fn post_empty(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<()> {
        let resp = self
            .auth_request(Method::POST, path)
            .await?
            .json(body)
            .send()
            .await?;
        Self::handle_empty_response(resp).await
    }

    /// Perform an authenticated PATCH request with a JSON body.
    pub(crate) async fn patch<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .auth_request(Method::PATCH, path)
            .await?
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// Perform an authenticated PUT request with a JSON body.
    pub(crate) async fn put<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .auth_request(Method::PUT, path)
            .await?
            .json(body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    /// Perform an authenticated PUT request with no response body.
    #[allow(dead_code)]
    pub(crate) async fn put_empty(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<()> {
        let resp = self
            .auth_request(Method::PUT, path)
            .await?
            .json(body)
            .send()
            .await?;
        Self::handle_empty_response(resp).await
    }

    /// Perform an authenticated DELETE request.
    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        let resp = self
            .auth_request(Method::DELETE, path)
            .await?
            .send()
            .await?;
        Self::handle_empty_response(resp).await
    }

    /// Handle a JSON response, mapping errors. (crate-public for use by query builder, etc.)
    pub(crate) async fn handle_response_pub<T: DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T> {
        Self::handle_response(resp).await
    }

    /// Handle a JSON response, mapping errors.
    async fn handle_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        if status.is_success() {
            let body = resp.json::<T>().await?;
            Ok(body)
        } else {
            Self::map_error(status, resp).await
        }
    }

    /// Handle a response that should have no body.
    async fn handle_empty_response(resp: reqwest::Response) -> Result<()> {
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Self::map_error(status, resp).await
        }
    }

    /// Map an error response to CopepodError::Api.
    async fn map_error<T>(status: StatusCode, resp: reqwest::Response) -> Result<T> {
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        Err(CopepodError::Api {
            status: status.as_u16(),
            code: body.get("code").and_then(|v| v.as_str()).map(String::from),
            message: body
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string(),
        })
    }
}
