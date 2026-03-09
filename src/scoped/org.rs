use crate::client::CopepodClient;

use super::app::ScopedAppClient;

/// Organization-scoped client helpers.
#[derive(Debug, Clone)]
pub struct ScopedOrgClient<'a> {
    client: &'a CopepodClient,
    org_id: String,
}

impl<'a> ScopedOrgClient<'a> {
    pub(crate) fn new(client: &'a CopepodClient, org_id: impl Into<String>) -> Self {
        Self {
            client,
            org_id: org_id.into(),
        }
    }

    /// Return the bound organization ID.
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    /// Bind an application ID and return an app-scoped helper.
    pub fn app(&self, app_id: impl Into<String>) -> ScopedAppClient<'a> {
        ScopedAppClient::new(self.client, self.org_id.clone(), app_id.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::CopepodClient;

    #[test]
    fn creates_scoped_app_client() {
        let client = CopepodClient::builder()
            .base_url("http://localhost:8090")
            .build()
            .unwrap();

        let app = client.org("org_123").app("app_456");
        assert_eq!(app.org_id(), "org_123");
        assert_eq!(app.app_id(), "app_456");
    }
}
