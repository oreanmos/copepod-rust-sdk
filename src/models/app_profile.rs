//! The signed-in app user's own display name.

use serde::{Deserialize, Serialize};

/// `{name}` of `GET`/`PATCH auth/{collection}/me/profile`; an empty stored
/// name reads as `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppProfile {
    #[serde(default)]
    pub name: Option<String>,
}
