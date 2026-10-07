---
name: oikonotes-dev
description: Working knowledge for the Oikonotes codebase (Rust, Leptos 0.8 SSR + islands, Axum, Tauri desktop, SQLite, Copepod cloud backend) - crate map, where code goes, the Make-based check/test commands, and the rules that cause silent failures (islands and routing, page data loading, web-mode storage via store traits, desktop CSR parity, leptos-daisyui/Tailwind 4, mobile). Load before writing, reviewing or planning any Oikonotes code, and whenever touching pages, islands, server functions, stores, Copepod usage, Tauri commands or UI styling in the oikonotes repo.
---

# Oikonotes Development

A personal notes/assistant app. Web mode: Leptos SSR + islands served by Axum,
persisting through Copepod. Desktop mode: the same UI as CSR inside Tauri,
persisting to local SQLite. Product source of truth:
`docs/spec/Personal_Assistant_Notes_App_Plan_v0.3.md`.

## Crate map

Package names are `oikonotes-<dir>` (tauri-app → `oikonotes-tauri`).

| Crate | Holds |
|---|---|
| `core` | framework-free domain types, parsers, wikilinks, and all store traits (`src/store/`) |
| `db` | SQLite via sqlx, migrations (`crates/db/migrations/`, auto-run), optional SQLCipher |
| `embeddings` | derived chunks + vectors, rebuildable |
| `ai` | `LlmProvider` abstraction (Mistral, Anthropic, OpenAI, Ollama) |
| `app` | thin composition root: server binary, router (`app_shell/routes`), hydrate/csr entry points, re-exports of the shell at the old paths, `cdylib` for cargo-leptos |
| `app-shell` | everything not in a domain crate: Leptos UI, pages, server functions, AI pipeline, stores |
| `app-kit` | what every app crate imports: `AppState`, config, errors, session, rate limiters, Copepod client, `Database`, `delegate_to_ssr!`, shared UI helpers; no domain code |
| `app-support` | small shared helpers (errors) for feature crates |
| `budget`, `investing`, `travel` | pure domain logic per product area |
| `investing-analytics`, `market-data` | native-only analytics and provider adapters (kept out of WASM) |
| `travel-service` | store-backed travel service between `travel` and `app` |
| `graph` | canvas knowledge-graph views |
| `public` | public blog/sharing surface |
| `media`, `render`, `push`, `vault-watch` | WebP transforms, markdown rendering, web push, desktop file watching |
| `tauri-app` | desktop shell: `src/commands/`, `generate_handler!` in `src/lib.rs` |

Pure logic goes in a domain crate, persistence behind a `core` store trait, and
Leptos/Axum orchestration in `app-shell` (or a domain crate once one exists).
When compile times regress, extract framework-free logic into a small crate
rather than growing `app-shell`. See `docs/plans/2026-10-05-app-crate-split.md`.

`crates/app-shell/src` (plus `app-kit` for the `AppState` type (`ssr/state`; the store wiring
`state/{web,desktop}.rs` stays in the shell until S2b), `config`, `copepod`, `db`,
`error`, `session`; the root `crates/app/src` holds `main.rs` and
`app_shell/routes`): `pages/` (route components), `ui/` (components and leaf
islands), `app_shell/` (document, layouts, navigation), `server/` (server
function implementations), `api.rs` (facade: server fns on web, Tauri IPC on
desktop), `tauri_ipc/`, `ssr/` (server-only: `state/`, `config/`, `copepod*/`,
`local_store/`, workers).

## Feature flags

`ssr` (shared server code) · `ssr-web` (hosted web server + UI) ·
`ssr-desktop` · `hydrate` (browser WASM, islands) · `csr` (desktop UI via Tauri
IPC). `ssr-web` and `hydrate` are separate build surfaces; bare `cargo check`
proves nothing about the app. rust-analyzer's all-feature view shows false
errors ("not in module tree", serde bounds on island props): trust the Make
targets, not the editor.

## Commands

Toolchain `nightly-2026-02-10`; Make targets use `CARGO_TARGET_DIR=target/local`.
Full guide: `docs/devex.md`.

```bash
make dev                      # launch.sh: Tailwind + cargo leptos watch
make check-affected           # narrowest checks for the current diff
make app-ssr | app-hydrate | app-csr | tauri-check
make test-affected FAST_TEST_FILTER=<name>
make fmt | fmt-check | clippy | test
make ci                       # full pre-merge set
make test-ui-focused E2E_SPEC=e2e/<spec>.spec.ts
scripts/devx/e2e-env.sh up --build | test <args> | down   # agents: per-worktree e2e server
npm run build:css             # Tailwind 4: npx @tailwindcss/cli -i crates/app/style/input.css -o crates/app/style/output.css --minify
```

Do not `cargo clean` during the edit loop; incremental state is the fast path.

## Rules that fail silently

Read the reference before working in its area.

- **Islands, routing, page data** → [references/leptos-islands.md](references/leptos-islands.md).
  Route content is never an `#[island]`; pages are `#[component]`s loading data
  with `Resource::new` + `<Suspense>`; only leaf widgets are islands. No
  `_`-prefixed variables inside `#[island]`; inner `#![allow(...)]` only; empty
  view is `().into_any()`.
- **Persistence and Copepod** → [references/stores-and-copepod.md](references/stores-and-copepod.md).
  In web mode `state.pool` is in-memory SQLite, wiped on every deploy: persistent
  user data goes through a store trait on `AppState` with both Copepod and local
  implementations. `scripts/check-state-pool-boundary.sh` enforces it.
- **Desktop parity** → [references/desktop-parity.md](references/desktop-parity.md).
  Shared UI calls backends through `crate::api::*` only; a new server domain
  needs `server/*`, `tauri_ipc/*`, a Tauri command and `generate_handler!`.
- **UI, styling, performance** → [references/ui-and-performance.md](references/ui-and-performance.md).
  `leptos_daisyui::prelude::*` first; mobile rules in `docs/mobile-conventions.md`;
  shell islands cache their calls; pollers respect `document.visibilityState`.
- **Tests and e2e** → [references/testing.md](references/testing.md).

## Always

- Human-in-the-loop for AI: suggestions carry a short reason and explicit
  apply/reject; no silent edits to notes, tasks or events.
- No `unwrap()`/`expect()` in runtime paths; `unsafe_code = "forbid"`.
- Files ≤300 lines target (500 hard), components ≤250, functions ≤50.
- No secrets in tracked files; `.env` stays local.
