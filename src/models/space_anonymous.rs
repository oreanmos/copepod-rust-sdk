//! Anonymous reads of a commons that its owner opened.

use serde::{Deserialize, Serialize};

use super::space::SpaceKind;

/// What `GET commons/{space_id}` shows anyone: the Space's id, name and kind
/// (always `commons`). The server sends no description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PublicCommons {
    pub id: String,
    pub name: String,
    pub kind: SpaceKind,
}

/// A signed download link for a file of a publicly readable commons record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PublicFileUrl {
    /// Server-relative path (`/api/platform/apps/{app}/files/signed/{key}?token=..&expires=..&anon=1`),
    /// relative to the Copepod base URL. Fetch it without credentials (see
    /// `CopepodClient::download_public_commons_file`); it answers 400 after
    /// `expires`, 401 once the app's master switch is off, and is served
    /// `no-store`.
    pub url: String,
    /// Expiry as Unix seconds.
    pub expires: u64,
}

/// Query of the anonymous commons record list. `expand` is refused by the
/// server, so it has no field here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonsRecordsQuery {
    /// Search text: a literal, ASCII case-insensitive contains-match over the
    /// collection's visible text fields (at most 100 characters).
    pub q: Option<String>,
    /// At most 8 comparisons, no `~` / `!~`.
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub fields: Option<String>,
    pub page: Option<u32>,
    /// At most 100 per page.
    pub per_page: Option<u32>,
}
