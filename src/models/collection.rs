use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A collection within an application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub collection_type: Option<String>,
    pub app_id: String,
    #[serde(default)]
    pub fields: Vec<CollectionField>,
    #[serde(default)]
    pub indexes: Vec<serde_json::Value>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// A field definition within a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}
