# UI, styling and performance

## Components and styling

- Build UI from `leptos_daisyui::prelude::*`; write a project wrapper only when
  composition is insufficient. Component and prop list:
  `docs/leptos-daisy-reference.md`; source at `~/Development/leptos-daisy`.
- `leptos-daisyui` is pinned by `rev` in the root `Cargo.toml` `[workspace.dependencies]` (HTTPS URL,
  never `branch`). To move it: change the rev, `cargo update -p leptos-daisyui`,
  then `make pin-check` and the app surface checks. Work on the library itself
  in its own repo and worktree.
- Tailwind 4 + daisyUI 5, config-less: source `crates/app/style/input.css`,
  output `crates/app/style/output.css` via `npm run build:css`
  (`make dev` watches it). There is no `tailwind.config.js`.
- Handle loading, error, empty and success states explicitly.
- Accessibility baseline: full keyboard navigation, visible focus, sufficient
  contrast, clear labels and action text.

## Mobile

Every page, island and shell component follows `docs/mobile-conventions.md`
(breakpoints, scroll containers, no horizontal overflow, tap targets, modals,
safe-area insets). Validate the touched routes with
`scripts/devx/e2e-env.sh test e2e/mobile-responsive.spec.ts -g "<route>"`; the
full spec runs at acceptance.

## Performance

- Shell islands in `AppLayout` (sidebar, navbar, pollers) run on every
  navigation: cache their API results in `localStorage` with a force-refresh
  on the relevant domain event. Uncached calls eat the browser's six
  connections per origin and block everything else.
- Pollers (`NotificationPoller`, `GlobalInboxRefreshIsland`) check
  `document.visibilityState` before any request.
- WASM size: the Docker build uses the `wasm-release` profile for the library
  (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`) and may use
  `ci-release` for the server binary; never switch the library to a speed
  profile. Assets are built with `--precompress` and `--split`.
- Lazy islands: write the pair
  `#[cfg_attr(feature = "client", island(lazy))]` +
  `#[cfg_attr(not(feature = "client"), island)]`, never a bare
  `#[island(lazy)]`. leptos_macro emits the lazy loader in every build, so a
  bare one compiles each island body twice in SSR; 125 of them took the
  ssr-web lib-test rustc from 8.5 to 19.5 GiB and OOM'd the 14 GiB release CI
  job (2026-10-06). `scripts/check-lazy-island-cfg.sh` (in `make ci`
  preflight) rejects a bare one.
