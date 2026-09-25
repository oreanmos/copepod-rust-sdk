# Stores, persistence and Copepod

## Modes

`AppState` (`crates/app/src/ssr/state/mod.rs`) holds `Arc<dyn XStore>` trait
objects chosen by `AppMode`:

- **Web mode** — `init_web_state` (`ssr/state/web.rs`): Copepod stores.
  `state.pool` / `legacy_sqlite_pool()` is an **in-memory** SQLite
  (`sqlite:file:webmode?mode=memory&cache=shared`), wiped on every restart and
  private to one replica. Only transient data (AI job queue, ingestion status)
  may live there.
- **Desktop mode** — `init_desktop_state` (`ssr/state/desktop.rs`): local SQLite
  stores (`OIKONOTES_DB_PATH`, default `data/oikonotes.sqlite` if present,
  otherwise the user data dir).

If you are about to write `state.pool` in a server function for user data,
stop: you need a store trait. `scripts/check-state-pool-boundary.sh` (in CI)
rejects new direct pool access; its baseline only shrinks.

## Adding a persistent data domain

1. Trait in `crates/core/src/store/<domain>.rs`.
2. `CopepodXStore` in `crates/app/src/ssr/copepod_store/<domain>.rs` over a
   Copepod collection (look at a similar domain first, e.g. `tasks.rs`,
   `shopping_lists.rs`).
3. `LocalXStore` in `crates/app/src/ssr/local_store/`, with a migration in
   `crates/db/migrations/` if it needs tables.
4. `pub x: Arc<dyn XStore>` on `AppState`; wire both init functions.
5. Server functions use `state.x`; UI reaches them through `crate::api`.
6. New env config is loaded in `ssr/config/` (`from_env.rs`, `copepod_config.rs`)
   and documented in `.env.copepod.example`.

## Copepod bridge

- `copepod-sdk` is pinned by full `rev` in `crates/app/Cargo.toml` (optional,
  behind `ssr`). Changing the pin or needing a new endpoint: load
  `oiko-contract`.
- All SDK use goes through `CopepodBridge` (`ssr/copepod.rs` + `ssr/copepod/`:
  auth, attachments, billing, email, files, tickets, objects, …). Never call
  `copepod_sdk::CopepodClient` from server functions or components. Keep the
  root file thin; add a submodule instead.
- `CopepodError` → `AppError` via the `From` impl in `ssr/copepod.rs`. Terminal
  auth errors (401, `refresh_family_not_found`, `ReuseDetected`,
  `FamilyInvalidated`) force re-login. `RAFT_LEADER_UNAVAILABLE_CODE` (503) is
  retryable.
- Realtime: `ssr/realtime.rs` listens to Copepod SSE for sync events.
- Calls take 100 ms – 30 s. Batch related calls, and run independent ones
  concurrently with `futures_util::future::join`/`try_join`, never sequentially.

## Configuration

`.env.copepod.example` lists the variables: `COPEPOD_BASE_URL`,
`COPEPOD_PUBLIC_BASE_URL`, `COPEPOD_ORG_ID`, `COPEPOD_APP_ID`,
`COPEPOD_API_KEY`, `COPEPOD_PLATFORM_TOKEN`, collection names
(`COPEPOD_AUTH_COLLECTION`, `_SYNC_`, `_ATTACHMENTS_`, `_TRAVEL_MEDIA_`,
`_RELEASE_`), `PUBLIC_BASE_URL`, `OIKONOTES_SESSION_COOKIE_*`. AI:
`MISTRAL_API_KEY`, `AI_PROCESSING_ENABLED`. Runbook:
`docs/copepod-mvp-integration-runbook.md`; smoke: `make copepod-smoke`.
