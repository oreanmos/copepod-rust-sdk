# Capacity and storage-quota errors

Two coded errors from Copepod (contract: copepod `08fbe913`,
`crates/copepod-api-shared/src/error.rs`, `docs/api-integration-reference.md` §22)
decode to typed `CopepodError` variants. Both carry their numbers at the top
level of the response body next to `code`, `error` and `message`.

| Error | Status | When | Variant | Accessors |
|---|---|---|---|---|
| `capacity_reached` | 409 | Signup (register, register-with-billing-intent) or a billing intent while the app is at a capacity limit | `CopepodError::CapacityReached { status, message, limit }` | `is_capacity_reached()`, `capacity_reached_limit()` |
| `storage_quota_exceeded` | 422 | An upload that would pass the user's `storage_gb` quota | `CopepodError::StorageQuotaExceeded { status, message, used_bytes, limit_bytes, file_bytes }` | `is_storage_quota_exceeded()`, `storage_quota_exceeded_bytes()` |

`limit` names the full capacity setting (for example `max_active_users`).
`api_status()` and `api_code()` answer for both variants, and the constants
`CAPACITY_REACHED_CODE` and `STORAGE_QUOTA_EXCEEDED_CODE` are exported.

**Do not retry.** The SDK never retries either error. Capacity clears only when
the operator raises a limit or a spot opens (show a waitlist); quota clears only
when the user frees space or upgrades.

**Fallback to `Api`.** The variant is used only when the status and code match
and every field is present with the right type (`limit` a string; the three byte
counts `u64`). Otherwise the error stays `CopepodError::Api { status, code,
message }`, as does any other 409 or 422.

**Older servers.** Servers before the capacity release never send these codes, so
the variants never appear and behaviour is unchanged.

**Compatibility.** `CopepodError` is not `#[non_exhaustive]`: code that matches
it exhaustively must add the two arms (or a wildcard).
