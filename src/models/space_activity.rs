//! The Space activity log. Contract: copepod `docs/api-integration-reference.md`,
//! "Activity".

use serde::{Deserialize, Serialize};

use super::space::SpaceRole;

/// What happened. Kinds the SDK does not know deserialize as `Other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpaceActivityKind {
    #[serde(rename = "item.added")]
    ItemAdded,
    #[serde(rename = "item.edited")]
    ItemEdited,
    #[serde(rename = "item.deleted")]
    ItemDeleted,
    #[serde(rename = "member.joined")]
    MemberJoined,
    #[serde(rename = "member.left")]
    MemberLeft,
    #[serde(rename = "member.removed")]
    MemberRemoved,
    #[serde(rename = "member.role_changed")]
    MemberRoleChanged,
    #[serde(rename = "invite.sent")]
    InviteSent,
    #[serde(rename = "comment.approved")]
    CommentApproved,
    #[serde(rename = "comment.pending")]
    CommentPending,
    /// A kind this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// Whether a person still belongs to the Space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceMembership {
    Member,
    Former,
    /// A state this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// A person in the activity log. `display_name` is null for a former member
/// and for a hidden or unset name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpacePerson {
    pub user_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub membership: SpaceMembership,
}

/// The record an activity item is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceActivityRecord {
    pub collection: String,
    pub record_id: String,
    #[serde(default)]
    pub deleted: bool,
}

/// One entry of the activity log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceActivityItem {
    pub kind: SpaceActivityKind,
    pub created: String,
    /// Null for `comment.*` events, which come from visitors.
    #[serde(default)]
    pub actor: Option<SpacePerson>,
    #[serde(default)]
    pub subject: Option<SpacePerson>,
    #[serde(default)]
    pub record: Option<SpaceActivityRecord>,
    #[serde(default)]
    pub role: Option<SpaceRole>,
    #[serde(default)]
    pub invite_email: Option<String>,
}

/// A page of the activity log, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceActivityPage {
    pub items: Vec<SpaceActivityItem>,
    /// Pass as `before` for the next page; null ends paging.
    #[serde(default)]
    pub next_before: Option<String>,
}
