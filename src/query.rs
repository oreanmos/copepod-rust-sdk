use serde_json::Value;

use crate::client::CopepodClient;
use crate::error::Result;
use crate::models::ListResult;

/// Fluent builder for querying records in a collection.
pub struct RecordQueryBuilder<'a> {
    client: &'a CopepodClient,
    path: String,
    filter: Option<String>,
    sort: Option<String>,
    expand: Option<String>,
    fields: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
}

impl<'a> RecordQueryBuilder<'a> {
    pub(crate) fn new(client: &'a CopepodClient, path: String) -> Self {
        Self {
            client,
            path,
            filter: None,
            sort: None,
            expand: None,
            fields: None,
            page: None,
            per_page: None,
        }
    }

    /// Set the filter expression.
    pub fn filter(mut self, filter: &str) -> Self {
        self.filter = Some(filter.to_string());
        self
    }

    /// Set the sort expression.
    pub fn sort(mut self, sort: &str) -> Self {
        self.sort = Some(sort.to_string());
        self
    }

    /// Set the expand expression for relation fields.
    pub fn expand(mut self, expand: &str) -> Self {
        self.expand = Some(expand.to_string());
        self
    }

    /// Set which fields to return.
    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = Some(fields.to_string());
        self
    }

    /// Set the page number.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of items per page.
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Build the query string from accumulated parameters.
    fn build_query(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(ref f) = self.filter {
            params.push(("filter".to_string(), f.clone()));
        }
        if let Some(ref s) = self.sort {
            params.push(("sort".to_string(), s.clone()));
        }
        if let Some(ref e) = self.expand {
            params.push(("expand".to_string(), e.clone()));
        }
        if let Some(ref f) = self.fields {
            params.push(("fields".to_string(), f.clone()));
        }
        if let Some(p) = self.page {
            params.push(("page".to_string(), p.to_string()));
        }
        if let Some(pp) = self.per_page {
            params.push(("per_page".to_string(), pp.to_string()));
        }
        params
    }

    /// Execute the query and return a paginated list of records.
    pub async fn list(self) -> Result<ListResult<Value>> {
        let query = self.build_query();
        let resp = self
            .client
            .auth_request(reqwest::Method::GET, &self.path)
            .await?
            .query(&query)
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }

    /// Get a single record by ID.
    pub async fn get_one(self, id: &str) -> Result<Value> {
        let path = format!("{}/{}", self.path, id);
        let query = self.build_query();
        let resp = self
            .client
            .auth_request(reqwest::Method::GET, &path)
            .await?
            .query(&query)
            .send()
            .await?;
        CopepodClient::handle_response_pub(resp).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_builder() -> RecordQueryBuilder<'static> {
        // Leak a client so we get a &'static reference — only in tests.
        let client = Box::leak(Box::new(
            CopepodClient::builder()
                .base_url("http://localhost")
                .build()
                .unwrap(),
        ));
        RecordQueryBuilder::new(client, "test".to_string())
    }

    #[test]
    fn test_query_builder_empty() {
        let builder = make_test_builder();
        let params = builder.build_query();
        assert!(params.is_empty());
    }

    #[test]
    fn test_query_builder_params() {
        let builder = make_test_builder()
            .filter("name='test'")
            .sort("-created")
            .expand("author")
            .fields("id,name")
            .page(2)
            .per_page(50);

        let params = builder.build_query();
        assert_eq!(params.len(), 6);
        assert_eq!(params[0], ("filter".to_string(), "name='test'".to_string()));
        assert_eq!(params[1], ("sort".to_string(), "-created".to_string()));
        assert_eq!(params[2], ("expand".to_string(), "author".to_string()));
        assert_eq!(params[3], ("fields".to_string(), "id,name".to_string()));
        assert_eq!(params[4], ("page".to_string(), "2".to_string()));
        assert_eq!(params[5], ("per_page".to_string(), "50".to_string()));
    }
}
