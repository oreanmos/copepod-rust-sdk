//! Share links, commons suggestions and takedown, plus the visitor view of a
//! Space.

use serde::{Deserialize, Serialize};

use super::space::{SpaceKind, SpaceWithRole};

/// What `GET spaces/{id}` shows a signed-in user who is not a member of a
/// commons: its name and kind only, never members or settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceVisitor {
    pub id: String,
    pub name: String,
    pub kind: SpaceKind,
}

/// The answer of [`crate::CopepodClient::get_space_view`]: the member shape
/// (it carries `role`) or the visitor shape of a commons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceView {
    Member(SpaceWithRole),
    Visitor(SpaceVisitor),
}

/// The literal `"space"` of [`ShareLinkScope::Space`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceKeyword {
    Space,
}

/// What a share link opens. Serialises as `"space"` or
/// `{"record_collection": .., "record_id": ..}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShareLinkScope {
    /// The whole Space.
    Space(SpaceKeyword),
    /// One record of a space-scoped collection.
    Record {
        record_collection: String,
        record_id: String,
    },
}

impl ShareLinkScope {
    /// The scope that opens the whole Space.
    pub fn whole_space() -> Self {
        Self::Space(SpaceKeyword::Space)
    }

    /// The scope that opens one record.
    pub fn record(collection: impl Into<String>, record_id: impl Into<String>) -> Self {
        Self::Record {
            record_collection: collection.into(),
            record_id: record_id.into(),
        }
    }
}

/// What a share link lets its holder do. Both levels read today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SharePermission {
    Read,
    Comment,
    /// A permission this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// A share link as listed. The token is never part of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceShareLink {
    pub id: String,
    pub space_id: String,
    pub scope: ShareLinkScope,
    pub permission: SharePermission,
    pub created_by: String,
    pub expires_at: String,
    #[serde(default)]
    pub revoked_at: Option<String>,
    pub created: String,
}

/// Body of `POST spaces/{space_id}/share-links`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLinkCreate {
    pub scope: ShareLinkScope,
    pub permission: SharePermission,
    /// RFC 3339. The server defaults to 30 days and allows at most 365.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Response of create and rotate: the only time the token is shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLinkIssued {
    pub token: String,
    /// API path of the link: `GET {url_path}/records/{collection}[/{id}]`.
    pub url_path: String,
    pub link: SpaceShareLink,
}

/// Query of the anonymous `shared/{token}/records/{collection}` list.
/// `expand` is refused through a link, so it has no field here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SharedRecordsQuery {
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub page: Option<u32>,
    /// At most 100 per page.
    pub per_page: Option<u32>,
}

/// A contributor's proposed record or change to a commons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceSuggestion {
    pub id: String,
    pub space_id: String,
    pub collection: String,
    /// The record a change targets; empty for a proposed new record.
    #[serde(default)]
    pub record_id: String,
    pub proposed: serde_json::Value,
    pub reason: String,
    /// The contributor, or `former_member` after their account was deleted.
    pub submitted_by: String,
    /// `pending`, `accepted`, `rejected` or `withdrawn`.
    pub status: String,
    #[serde(default)]
    pub decided_by: String,
    #[serde(default)]
    pub decided_at: Option<String>,
    #[serde(default)]
    pub decision_note: String,
    /// The record an accepted suggestion created or changed.
    #[serde(default)]
    pub applied_record_id: String,
    /// The revision a change was proposed against, when the collection has a
    /// `revision_field`.
    #[serde(default)]
    pub base_rev: String,
    #[serde(default)]
    pub applied_at: Option<String>,
    pub created: String,
}

/// Body of `POST spaces/{space_id}/suggestions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestionCreate {
    pub collection: String,
    /// The record to change; omit for a new record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    /// Schema fields of the collection, at most 16 KB serialised.
    pub proposed: serde_json::Value,
    /// Required, at most 1000 characters.
    pub reason: String,
}

/// Options of [`crate::CopepodClient::list_space_suggestions`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SuggestionListOptions {
    /// `pending`, `accepted`, `rejected` or `withdrawn`.
    pub status: Option<String>,
    /// 1 to 200; the server defaults to 50.
    pub limit: Option<u32>,
    /// The `next` of the previous page.
    pub after: Option<String>,
}

/// One page of suggestions; pass `next` as `after` for the following one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestionPage {
    pub items: Vec<SpaceSuggestion>,
    #[serde(default)]
    pub next: Option<String>,
}

/// Response of accepting a suggestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestionAccepted {
    pub suggestion: SpaceSuggestion,
    /// The record the suggestion created or changed.
    pub record: serde_json::Value,
}

/// Response of a takedown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakedownResult {
    pub taken_down: bool,
    pub record_id: String,
    /// The record's author, or null.
    #[serde(default)]
    pub created_by: Option<String>,
}
