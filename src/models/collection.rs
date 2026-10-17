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
    /// Collection options such as `revision_field` (the `If-Match` revision).
    #[serde(default)]
    pub options: Option<serde_json::Value>,
    /// `"space"` for collections whose rows live in a Space (`X-Space-Id`).
    #[serde(default)]
    pub shard_target: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::Collection;

    #[test]
    fn decodes_options_and_shard_target_when_present_or_absent() {
        let base = r#"{"id":"c","name":"notes","app_id":"a",
            "created":"2026-10-01T00:00:00Z","updated":"2026-10-01T00:00:00Z""#;
        let old: Collection = serde_json::from_str(&format!("{base}}}")).unwrap();
        assert!(old.options.is_none() && old.shard_target.is_none());
        let new: Collection = serde_json::from_str(&format!(
            r#"{base},"options":{{"revision_field":"rev"}},"shard_target":"space"}}"#
        ))
        .unwrap();
        assert_eq!(new.options.unwrap()["revision_field"], "rev");
        assert_eq!(new.shard_target.as_deref(), Some("space"));
    }
}
