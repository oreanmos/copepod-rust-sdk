//! The credit ledger of the signed-in app user (`me/credits*`). One ledger
//! covers hosted AI and metered connectors. App-user bearer token.

use crate::client::CopepodClient;
use crate::error::{CopepodError, Result};
use crate::models::{
    CreditCheckRequest, CreditCheckResponse, CreditReportRequest, CreditReportResponse,
    CreditStatus,
};

fn base(org_id: &str, app_id: &str, collection: &str) -> String {
    format!("api/platform/orgs/{org_id}/apps/{app_id}/auth/{collection}/me/credits")
}

fn validate_kind(kind: &str) -> Result<()> {
    let named = |prefix: &str| {
        kind.strip_prefix(prefix)
            .is_some_and(|name| !name.is_empty())
    };
    if kind == "ai" || named("ai:") || named("connector:") {
        Ok(())
    } else {
        Err(CopepodError::InvalidArgument(
            "kind must be `ai`, `ai:<name>` or `connector:<name>`".into(),
        ))
    }
}

impl CopepodClient {
    /// This month's credit allowance, top-ups and spend by kind.
    pub async fn get_current_credits(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
    ) -> Result<CreditStatus> {
        self.get(&base(org_id, app_id, collection)).await
    }

    /// Check that enough credits are left before spending.
    ///
    /// Fails with 402 `credits_exhausted` when not
    /// ([`CopepodError::credits_exhausted_details`]).
    pub async fn check_current_credits(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &CreditCheckRequest,
    ) -> Result<CreditCheckResponse> {
        validate_kind(&body.kind)?;
        self.post(&format!("{}/check", base(org_id, app_id, collection)), body)
            .await
    }

    /// Report a spend. A retry with the same `request_id` returns the first
    /// answer. Fails with 409 `credits_exhausted` when nothing is left, 400
    /// for an unpriced or inactive connector, 503 `credits_upgrading` during a
    /// rolling upgrade.
    pub async fn report_current_credits(
        &self,
        org_id: &str,
        app_id: &str,
        collection: &str,
        body: &CreditReportRequest,
    ) -> Result<CreditReportResponse> {
        let body = CreditReportRequest {
            kind: body.kind.trim().to_string(),
            ..body.clone()
        };
        validate_kind(&body.kind)?;
        if body.request_id.trim().is_empty() {
            return Err(CopepodError::InvalidArgument(
                "request_id must not be empty".into(),
            ));
        }
        self.post(
            &format!("{}/report", base(org_id, app_id, collection)),
            &body,
        )
        .await
    }
}
