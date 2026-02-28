use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::{DsarRequest, ExportJob, ListResult, RetentionPolicy};

impl CopepodClient {
    // --- Retention Policies ---

    /// List retention policies for an organization.
    pub async fn list_retention_policies(
        &self,
        org_id: &str,
    ) -> Result<ListResult<RetentionPolicy>> {
        self.get(&format!("api/platform/orgs/{}/retention", org_id))
            .await
    }

    /// Create a retention policy.
    pub async fn create_retention_policy(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<RetentionPolicy> {
        self.post(
            &format!("api/platform/orgs/{}/retention", org_id),
            body,
        )
        .await
    }

    /// Update a retention policy.
    pub async fn update_retention_policy(
        &self,
        org_id: &str,
        policy_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<RetentionPolicy> {
        self.patch(
            &format!(
                "api/platform/orgs/{}/retention/{}",
                org_id, policy_id
            ),
            body,
        )
        .await
    }

    /// Delete a retention policy.
    pub async fn delete_retention_policy(
        &self,
        org_id: &str,
        policy_id: &str,
    ) -> Result<()> {
        self.delete(&format!(
            "api/platform/orgs/{}/retention/{}",
            org_id, policy_id
        ))
        .await
    }

    // --- Data Exports ---

    /// List data export jobs.
    pub async fn list_exports(&self, org_id: &str) -> Result<ListResult<ExportJob>> {
        self.get(&format!("api/platform/orgs/{}/exports", org_id))
            .await
    }

    /// Create a data export job.
    pub async fn create_export(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<ExportJob> {
        self.post(&format!("api/platform/orgs/{}/exports", org_id), body)
            .await
    }

    /// Get the status of a data export job.
    pub async fn get_export(&self, org_id: &str, export_id: &str) -> Result<ExportJob> {
        self.get(&format!(
            "api/platform/orgs/{}/exports/{}",
            org_id, export_id
        ))
        .await
    }

    // --- DSAR (Data Subject Access Requests) ---

    /// List DSAR requests.
    pub async fn list_dsar_requests(
        &self,
        org_id: &str,
    ) -> Result<ListResult<DsarRequest>> {
        self.get(&format!("api/platform/orgs/{}/dsar", org_id))
            .await
    }

    /// Create a DSAR request.
    pub async fn create_dsar_request(
        &self,
        org_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<DsarRequest> {
        self.post(&format!("api/platform/orgs/{}/dsar", org_id), body)
            .await
    }

    /// Get the status of a DSAR request.
    pub async fn get_dsar_request(
        &self,
        org_id: &str,
        request_id: &str,
    ) -> Result<DsarRequest> {
        self.get(&format!(
            "api/platform/orgs/{}/dsar/{}",
            org_id, request_id
        ))
        .await
    }
}
