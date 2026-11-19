//! Share-link comments: the visitor view, the member view and the owner's
//! decision. Contract: copepod `docs/api-integration-reference.md`, "Link
//! comments".

use serde::{Deserialize, Serialize};

use super::space_sharing::SharePermission;

/// What a share link opens, as `GET shared/{token}` reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SharedLinkScope {
    /// The whole Space.
    Space,
    /// One record of a space-scoped collection.
    Record {
        collection: String,
        record_id: String,
    },
    /// A scope this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// The answer of `GET shared/{token}`: no ids, creator or token hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedLinkInfo {
    pub permission: SharePermission,
    pub scope: SharedLinkScope,
    pub expires_at: String,
}

/// An approved comment as a link visitor sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedComment {
    pub id: String,
    #[serde(default)]
    pub author_name: Option<String>,
    pub body: String,
    pub created: String,
}

/// Who to tell about a new comment; only an API-key caller gets it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedCommentNotify {
    pub space_id: String,
    pub collection: String,
    pub record_id: String,
    pub owner_user_ids: Vec<String>,
    /// No other comment on the record in the previous 60 minutes.
    pub first_in_hour: bool,
}

/// The answer of `POST shared/{token}/records/{collection}/{id}/comments`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedCommentCreated {
    pub id: String,
    /// Always `pending` on creation.
    pub status: SpaceCommentStatus,
    #[serde(default)]
    pub notify: Option<SharedCommentNotify>,
}

/// Where a comment stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceCommentStatus {
    Pending,
    Approved,
    Rejected,
    Removed,
    /// A status this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// A comment as a Space member sees it (`GET spaces/{id}/comments`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceComment {
    pub id: String,
    pub collection: String,
    pub record_id: String,
    #[serde(default)]
    pub author_name: Option<String>,
    pub body: String,
    pub status: SpaceCommentStatus,
    pub created: String,
    #[serde(default)]
    pub decided_at: Option<String>,
}

/// An owner's decision on a comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentDecision {
    /// A pending comment becomes visible.
    Approve,
    /// A pending comment is refused.
    Reject,
    /// An approved comment is taken down.
    Remove,
}

/// The answer of `POST spaces/{id}/comments/{comment_id}/decision`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceCommentDecided {
    pub id: String,
    pub status: SpaceCommentStatus,
}
