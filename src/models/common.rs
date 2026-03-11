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

/// Simple list response wrapping an items array (no pagination).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsResponse<T> {
    pub items: Vec<T>,
}
