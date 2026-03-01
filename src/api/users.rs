use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{ListResult, User};

impl CopepodClient {
    /// List all platform users.
    pub async fn list_users(&self) -> Result<ListResult<User>> {
        self.get("api/platform/users").await
    }

    /// Get a platform user by ID.
    pub async fn get_user(&self, id: &str) -> Result<User> {
        self.get(&format!("api/platform/users/{}", id)).await
    }

    /// Create a new platform user.
    pub async fn create_user(&self, body: &impl serde::Serialize) -> Result<User> {
        self.post("api/platform/users", body).await
    }

    /// Update a platform user.
    pub async fn update_user(&self, id: &str, body: &impl serde::Serialize) -> Result<User> {
        self.patch(&format!("api/platform/users/{}", id), body).await
    }

    /// Reset a platform user's password (admin).
    pub async fn reset_password(
        &self,
        id: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value> {
        self.post(
            &format!("api/platform/users/{}/reset-password", id),
            body,
        )
        .await
    }
}
