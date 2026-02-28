use serde::{Deserialize, Serialize};

/// Paginated list response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult<T> {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub items: Vec<T>,
}
