//! Decoding of Copepod's typed capacity and quota error bodies.

use crate::error::{CopepodError, CAPACITY_REACHED_CODE, STORAGE_QUOTA_EXCEEDED_CODE};

/// Map `(409, capacity_reached)` and `(422, storage_quota_exceeded)` bodies to
/// their typed variants. Returns `None` for every other response, and when a
/// required field is missing or mistyped, so the caller falls back to
/// [`CopepodError::Api`].
pub(crate) fn typed_error(status: u16, bytes: &[u8]) -> Option<CopepodError> {
    if status != 409 && status != 422 {
        return None;
    }
    let body: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let code = body.get("code")?.as_str()?;
    let message = body
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or(code)
        .to_string();
    match (status, code) {
        (409, CAPACITY_REACHED_CODE) => Some(CopepodError::CapacityReached {
            status,
            message,
            limit: body.get("limit")?.as_str()?.to_string(),
        }),
        (422, STORAGE_QUOTA_EXCEEDED_CODE) => Some(CopepodError::StorageQuotaExceeded {
            status,
            message,
            used_bytes: body.get("used_bytes")?.as_u64()?,
            limit_bytes: body.get("limit_bytes")?.as_u64()?,
            file_bytes: body.get("file_bytes")?.as_u64()?,
        }),
        _ => None,
    }
}
