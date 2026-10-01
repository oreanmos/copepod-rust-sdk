---
name: copepod-dev
description: Working knowledge for the Copepod platform server (Rust, Axum 0.8, SQLite control plane with app/user shard data plane, Raft/LiteFS replication, Leptos 0.8 CSR admin UI) - crate map, how to add an endpoint across domain crates and the split router, auth and tenant-isolation rules, the 500-line cap, validation commands, CI and plan conventions. Load before writing, reviewing or planning any code in the copepod repo - handlers, routes, models, persistence, auth, storage, deployment, or the admin UI.
---

# Copepod Development

Copepod is a multi-tenant backend platform (orgs → apps → app users) serving
auth, collections/records, files, realtime, billing, email, tickets,
deployments and more. Its first production consumer is Oikonotes, through
copepod-rust-sdk: API changes follow `oiko-contract`.

## Crate map

| Crate | Holds |
|---|---|
| `copepod` | binary: CLI (`serve`, `migrate`, `import`), server bootstrap, metrics sampler; features `cache`, `docker`, `kubernetes` |
| `copepod-api` | router assembly only: `src/router.rs` (middleware, merges) + `src/router/<topic>.rs`; `src/handlers/mod.rs` re-exports the domain crates |
| `copepod-api-billing` / `-comms` / `-data` / `-deploy` / `-platform` | HTTP handlers by domain, in `src/handlers/` |
| `copepod-api-shared` | extractors, errors, middleware, state, metrics, auth tokens, platform session, raft writes, stream tickets |
| `copepod-core` | business rules and services |
| `copepod-db` | persistence: control plane, shards, backup, import, FTS, logs |
| `copepod-models` | shared API and domain types |
| `copepod-auth` | token store, envelopes, secrets, TOTP, VAPID |
| `copepod-storage` | local and remote (S3/SFTP) storage, transforms |
| `copepod-replication` | LiteFS and Raft (openraft) modes |
| `copepod-ui` | admin UI, Leptos CSR via Trunk |

Out of workspace: `k8s/operator` and `tests/harness` (own Cargo manifests,
checked in CI with `--manifest-path`).

## Adding or changing an endpoint

1. Handler in the owning domain crate: `crates/copepod-api-<domain>/src/handlers/<area>.rs`.
2. Route in the matching `crates/copepod-api/src/router/<topic>.rs`
   (health_auth, orgs_apps, collections_records, files_email, billing,
   iam_platform, app_services, operations, cluster, deployments, app_comms,
   raft), referenced as `handlers::<area>::<fn>`.
3. Types in `copepod-models`; business rules in `copepod-core`; queries in
   `copepod-db`; new extractors in `copepod-api-shared`.
4. Tests for success, auth failure and isolation.
5. Externally visible? Update `docs/api-integration-reference.md` and the
   OpenAPI surface (`python3 scripts/generate_openapi_surface.py`, then
   `--check`), and continue with the SDK per `oiko-contract`.

## Auth and isolation

- Admin UI: HttpOnly platform auth cookies, same-origin credentialed requests.
- External clients and app-scoped data APIs: `Authorization: Bearer <token>`;
  API-key endpoints use `X-API-Key` with scopes.
- Never mix platform auth and app-user auth.
- Every query and API path preserves org, app and user isolation; keep control
  plane and data plane separate.
- Refresh rotation, logout, stream tickets, app binding, CORS, webhooks and
  destructive actions need regression tests.
- No secrets in code, docs, logs, snapshots or tests.

## Admin UI

`crates/copepod-ui` is CSR only: no SSR, no islands, no `#[server]`. HTTP via
`gloo-net` with same-origin credentials. `leptos_daisyui::prelude::*`
components, Tailwind 4 + daisyUI 5, mobile-first. Themes are `ingrained` and
`ingrained-light` in `style/themes.css`, contrast-checked by
`node scripts/ui-theme-contrast.mjs --check`; pages use only daisyUI semantic
colours (`python3 scripts/check-ui-semantic-colors.py --check`). Fonts are
self-hosted IBM Plex in `crates/copepod-ui/fonts/`. Served at `/_/`
(`Trunk.toml`). After UI dependency changes run
`scripts/check-ui-css-versions.sh`. Verify UI changes in a browser (Playwright:
`npm run test:e2e:admin-ui`, `:launchpad`, `:webhook`). New routes and
components need an entry in `docs/design-review/inventory.json`
(`python3 scripts/check-design-inventory.py --add-missing`, then `--write-md`).

## Size cap

500 lines per Rust source file. Oversized legacy files are inventoried in
`docs/production-hardening-open-items.md`; do not grow them, and split by
concern when you must touch one heavily (the router split is the model).

## Commands

Toolchain `nightly-2026-02-10`, edition 2024, mold linker (`.cargo/config.toml`).
Prefix cargo with `CARGO_TARGET_DIR=target/local`.

```bash
cargo check -p <crate>                  # while editing
cargo fmt --all -- --check
cargo clippy --locked --workspace --exclude copepod-ui --all-targets -- -D warnings
cargo test --locked --workspace --exclude copepod-ui
cargo check -p copepod-ui && npm run build:ui:css
python3 scripts/generate_openapi_surface.py --check
cargo run -p copepod -- serve [--host H --port P]   # local server
scripts/validate-p0-release-gates.sh    # release-impacting changes
```

CI is Forgejo only (`.forgejo/workflows/`; GitHub Actions were removed
2026-09-30). `scripts/validate-p0-release-gates.sh` runs in the production
`release-gates` job and covers the deployment-safety scripts, the Kubernetes
feature clippy (`-p copepod --features kubernetes,remote-sftp`), operator and
harness checks and `npm audit`. Deploys: Forgejo `staging` and production
pushes. There is no PR gate and no scheduled audit or uptime probe. Never
push or deploy unless asked. If a gate cannot run, report the command, reason
and residual risk.

## Environments

| Host | Role |
|---|---|
| `10.0.40.10` (`build01`) | Forgejo mirror, container registry `build.ulev.org/cesarli`, forgejo-runner |
| `10.0.20.8` (`dev-server-01`) | staging k3s: CopepodCluster `dev-cluster` |
| `10.0.30.8` (`prod-server-01`) | production k3s: CopepodCluster `prod-cluster`, public via cloudflared |

SSH as the owner's user; `kubectl` works on the k3s nodes. Everything runs in
namespace `copepod`; Oikonotes is a Copepod-deployed app
(`a-<id>-oikonotes`). Images are digest-pinned from `build.ulev.org`. Treat
staging and production as read-only (`get`, `describe`, `logs`) unless the
owner asks for a change in that session; never `apply`, `delete`, restart or
exec into production on your own initiative.

## Docs and plans

Plans: `docs/plans/YYYY-MM-DD-<topic>-design.md` (+ `-plan.md` for the
implementation plan). Architecture: `docs/copepod-architecture.md`;
operations: `docs/raft-operations.md`, `docs/deployment-*.md`,
`docs/production-validation-runbook.md`. Read these only when changing that
subsystem's contract.
