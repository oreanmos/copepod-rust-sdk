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

    /// Delete a platform user.
    pub async fn delete_user(&self, id: &str) -> Result<()> {
        self.delete(&format!("api/platform/users/{}", id)).await
    }
}
