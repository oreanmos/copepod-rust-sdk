---
name: copepod-sdk-dev
description: Working knowledge for copepod-rust-sdk, the typed Rust client (crate `copepod-sdk`) for the Copepod API used by Oikonotes - module layout, the exact pattern for adding an endpoint method, auth and token refresh, error variants, wiremock tests, docs, and how Oikonotes pins it. Load before writing, reviewing or planning code in the copepod-rust-sdk repo, or when Oikonotes needs an SDK method that does not exist yet.
---

# copepod-rust-sdk Development

Crate `copepod-sdk` v0.1.0, edition 2021, standalone (`[workspace]` stub, no
toolchain file, no CI, no tags). Oikonotes pins it by full commit SHA from
`https://github.com/oreanmos/copepod-rust-sdk.git`; see `oiko-contract` for the
push gate and pin bump.

## Layout

| Path | Holds |
|---|---|
| `src/client.rs` | `CopepodClient` + builder (base URL, token, refresh token, API key, `auto_refresh`, custom `reqwest::Client`); `request()`, `handle_response_pub()`, `map_error()` |
| `src/auth.rs` | `TokenPair`, `TokenStore` (refresh 60 s before expiry) |
| `src/error.rs` | `CopepodError`: `Http`, `Api{status,code,message}`, `Auth`, `InvalidArgument`, `Deserialize`, `Url`, `Sse`, `Io`; `is_raft_leader_unavailable()`, `raft_retry_after()` |
| `src/api/<domain>.rs` | one `impl CopepodClient` block per domain with the endpoint methods |
| `src/models/<domain>.rs` | request/response types, re-exported from `models` |
| `src/scoped/` | ergonomic wrappers that close over org/app IDs (`client.app(org, app).email()`) |
| `src/query.rs`, `src/realtime.rs` | query builders; SSE streams |

Dependencies: `reqwest` 0.12 (rustls, json, stream, multipart), `tokio` (sync),
`serde`, `thiserror` 2, `chrono`, `eventsource-stream`. Keep it lean: no new
dependency without a clear need.

## Adding an endpoint

Model on `create_app_billing_intent_idempotent` (`src/api/billing.rs`,
`tests/billing_intent_recovery.rs`, `docs/billing-intent-recovery.md`):

1. Add the method to the existing block in `src/api/<domain>.rs`; plain ID
   arguments plus a request model, returning `Result<ResponseModel>`.
2. Validate arguments first and return `CopepodError::InvalidArgument` before
   any network call.
3. `self.request(Method::POST, &format!("api/platform/orgs/{org_id}/apps/{app_id}/…"))`,
   then `.json(body).send().await?` and `CopepodClient::handle_response_pub(response).await`.
4. Types: extend `src/models/<domain>.rs`; derive serde; optional fields as
   `Option` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
   where the server treats them as optional.
5. Auth: bearer via the shared `TokenStore` for user/platform calls. API-key
   endpoints send `X-API-Key` only and must not send `Authorization`; test that.
6. Add a scoped wrapper when the area already has one in `src/scoped/`.
7. Doc-comment the public method: what it calls, auth, errors.

## Tests

All tests use `wiremock` (`MockServer`, `Mock::given(method(..)).and(path(..))`,
header and `body_json` matchers, `.expect(n)`); none need a live server.
Put a new contract in its own `tests/<topic>.rs`; cover the success path, the
error mapping, and the auth header kind. Add `docs/<topic>.md` for non-obvious
semantics (idempotency keys, conflicts, behaviour against older servers). When
a newer SDK may meet an older server, fail clearly rather than silently falling
back (see "reject unsupported intent recovery servers").

## Commands

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test                       # or: cargo test --test <file>
```

`Cargo.lock` is gitignored (library crate).
