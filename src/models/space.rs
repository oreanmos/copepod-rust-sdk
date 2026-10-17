//! Spaces: shared data containers with members and roles.

use serde::{Deserialize, Serialize};

/// Whether a Space is closed to its members or an open commons.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceKind {
    /// Only members see it. The only kind that can be created today.
    #[default]
    Private,
    Commons,
    /// A kind this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// A member's role in a Space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpaceRole {
    Owner,
    Editor,
    Viewer,
    Contributor,
    Moderator,
    /// A role this SDK does not know yet.
    #[serde(other)]
    Other,
}

/// A shared data container with its own database file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub app_id: String,
    pub name: String,
    pub kind: SpaceKind,
    pub owner_user_id: String,
    #[serde(default)]
    pub anonymous_read: bool,
    pub created: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
}

/// A Space with the caller's role in it (`GET spaces`, accept invite).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceWithRole {
    #[serde(flatten)]
    pub space: Space,
    pub role: SpaceRole,
}

/// A user's membership of a Space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceMember {
    pub space_id: String,
    pub user_id: String,
    pub role: SpaceRole,
    pub added_by: String,
    pub created: String,
}

/// Body of `POST spaces`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceCreate {
    pub name: String,
    /// Only [`SpaceKind::Private`] is accepted for now.
    #[serde(default)]
    pub kind: SpaceKind,
}

/// A pending invitation to join a Space. The emailed token is never returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceInvite {
    pub id: String,
    pub space_id: String,
    pub email: String,
    pub role: SpaceRole,
    #[serde(default)]
    pub invited_by: String,
    /// `pending`, `accepted` or `revoked`.
    pub status: String,
    pub expires_at: String,
    #[serde(default)]
    pub created: String,
}

/// Body of `POST spaces/{space_id}/invites`. `role` is `owner`, `editor` or
/// `viewer` for now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceInviteCreate {
    pub email: String,
    pub role: SpaceRole,
}

/// Details of a 409 `revision_conflict` on an `If-Match` update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisionConflict {
    /// The stored revision (a JSON number).
    pub current_rev: serde_json::Value,
}

/// Header and precondition options for [`crate::CopepodClient::update_record_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordUpdateOptions {
    /// Send `X-Space-Id` (required for space-scoped collections).
    pub space_id: Option<String>,
    /// Send `If-Match: <rev>`; the collection must declare
    /// `options.revision_field`.
    pub if_match: Option<String>,
}
