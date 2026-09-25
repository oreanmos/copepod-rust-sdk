# Desktop (Tauri) parity

The desktop app renders the same Leptos UI as CSR inside Tauri and talks to the
backend over IPC instead of HTTP.

- Route/page and shared UI code calls the backend only through `crate::api::*`
  (`crates/app/src/api.rs`): it re-exports `crate::server::<domain>` under
  `ssr`/`hydrate` and `crate::tauri_ipc::<domain>` under `csr`. Never import
  `crate::server::*` directly from shared UI.
- A new server domain used by pages needs every layer: `server/<domain>`,
  `tauri_ipc/<domain>`, the Tauri command in `crates/tauri-app/src/commands/`,
  its entry in `tauri::generate_handler!` (`crates/tauri-app/src/lib.rs`), and
  the `api.rs` re-export pair. `npm run test:contract:ipc`
  (`scripts/quality/tauri-ipc-contract.mjs`) checks the wiring.
- No hardcoded `/api/...` fetches in CSR paths; gate HTTP-only behaviour to
  `hydrate` and provide the IPC fallback.
- No browser-only APIs in code shared with Tauri surfaces.
- Checks: `make app-csr`, `make tauri-check`. Desktop e2e (WebdriverIO against
  the release Tauri binary): `make e2e-desktop`, specs in `e2e-desktop/`.
- Build and packaging notes: `docs/desktop-build.md`.
