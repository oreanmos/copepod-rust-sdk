---
name: oiko-contract
description: The cross-repo API contract between the Copepod server, copepod-rust-sdk and Oikonotes - deciding whether a feature needs a server change at all, additive vs breaking changes, the exact server → OpenAPI surface → SDK → push → Oikonotes pin-bump → bridge/store sequence, verifying against an unpushed local SDK, and deploy order. Load whenever a Copepod endpoint, request/response shape, error code, auth scope or realtime event is added or changed, when Oikonotes needs something the SDK does not expose, when bumping the copepod-sdk rev, or when an SDK/server mismatch is suspected.
---

# Oiko API Contract

Copepod (server) → copepod-rust-sdk (typed client) → Oikonotes (pins the SDK by
git rev, wraps it in `CopepodBridge`). A contract change is only done when all
three agree and each is verified.

## First: does it need a server change?

Usually not. Copepod's generic collections, records, files, signed URLs and
realtime cover ordinary app persistence: a new Oikonotes data domain is a store
trait plus a `CopepodXStore` over a collection (see `oikonotes-dev`). Change the
server only for a missing platform capability: a new auth/billing/email/ticket
flow, server-side validation or aggregation, or a performance need records
cannot meet.

## Change class

- **Additive** (new endpoint, new optional field, new error code consumers can
  ignore): preferred. Existing SDK and Oikonotes code keeps compiling and
  working unchanged.
- **Breaking** (renamed/removed field, changed type, removed endpoint, stricter
  validation): keep the old shape serving alongside the new for one release
  where possible; server, SDK and Oikonotes land as one coordinated build, and
  the Oikonotes pin is held until all three are ready.

`copepod/docs/consumer-compatibility.md` describes this policy and the
verification that exists today: the SDK has no release tags (Oikonotes pins a
full commit SHA) and its tests are wiremock-only.

## The sequence

### 1. Server slice (copepod)

- Handler in the owning domain crate: `crates/copepod-api-<billing|comms|data|deploy|platform>/src/handlers/<area>.rs`.
- Route in the matching `crates/copepod-api/src/router/<topic>.rs`.
- Request/response types in `copepod-models`; new auth/context extractors in
  `copepod-api-shared`.
- Tests: success, auth failure, and tenant/org/app isolation.
- Update `docs/api-integration-reference.md`, then regenerate and check the
  surface: `python3 scripts/generate_openapi_surface.py` and
  `python3 scripts/generate_openapi_surface.py --check`. Commit both.
- Stable, machine-readable error codes for anything a client must branch on
  (pattern: `raft_leader_unavailable`).

### 2. SDK slice (copepod-rust-sdk)

- Method in the existing `impl CopepodClient` block of `src/api/<domain>.rs`;
  types in `src/models/<domain>.rs`; an ergonomic wrapper in `src/scoped/` when
  the call is org/app scoped and a scoped wrapper for that area exists.
- Validate arguments client-side first (`CopepodError::InvalidArgument`).
- Use the right auth: bearer tokens for user/platform calls; `X-API-Key` only
  (no bearer) for API-key endpoints.
- A wiremock test in `tests/<topic>.rs` that asserts method, path, auth header
  kind and body, plus the error path.
- For non-obvious semantics (idempotency, retries, fallbacks, older servers),
  add `docs/<topic>.md`. When a newer SDK may meet an older server, fail clearly
  rather than silently falling back.

### 3. Push gate

Oikonotes resolves `copepod-sdk` from
`https://github.com/oreanmos/copepod-rust-sdk.git` at a pinned `rev`, so the SDK
commit must be on `origin` before Oikonotes can commit a pin to it. Pushing
the merged SDK `main` to `origin/next` is pre-authorized (`oiko-worktree`, Two
machines). A rev on `next` resolves like any other, so pin it. Pushing SDK
`main` stays the owner's call.

Before the push, an Oikonotes worktree can compile against a local SDK checkout
with a command-line patch (no tracked file changes except `Cargo.lock`):

```bash
cd <oikonotes-worktree> && CARGO_TARGET_DIR=target/local cargo check -p oikonotes-app \
  --no-default-features --features ssr-web \
  --config 'patch."https://github.com/oreanmos/copepod-rust-sdk.git".copepod-sdk.path="<abs sdk checkout>"'
```

This rewrites `Cargo.lock` to a path source: run `git checkout -- Cargo.lock`
afterwards and never commit it (CI uses `--locked`).

### 4. Oikonotes slice

1. Set the new full SHA in the root `Cargo.toml` (`[workspace.dependencies]`,
   `copepod-sdk = { git = …, rev = "<sha>" }`; the app crates inherit it), then
   `CARGO_TARGET_DIR=target/local cargo update -p copepod-sdk` and `make app-ssr`.
2. Call the SDK only through `CopepodBridge` (`crates/app-kit/src/ssr/copepod.rs`
   and `ssr/copepod/`); map errors through the existing `From<CopepodError>`.
3. Consume it in the store (`ssr/copepod_store/`), keep the local store
   (`ssr/local_store/`) behaviour coherent for desktop mode, and expose it to UI
   through server functions / `crate::api`.
4. Live check when useful: `make copepod-smoke` (needs `COPEPOD_*` env) or run
   Oikonotes web mode against a local Copepod (`cargo run -p copepod -- serve`).

### 5. Deploy order

Server first, then the Oikonotes release that uses it. Record the required
order in the build's finish report; deploy only when the owner asks.

## Diagnosing a mismatch

Compare the three views of one endpoint: the route and handler in copepod, the
path/body/auth in the SDK method and its wiremock test, and the pinned rev in
`oikonotes/Cargo.toml` `[workspace.dependencies]` (`git -C copepod-rust-sdk log --oneline <rev>..main`
shows what Oikonotes is missing).
