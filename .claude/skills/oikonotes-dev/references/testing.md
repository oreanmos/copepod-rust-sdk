# Tests and e2e

Release methodology, fixtures and product contract:
`docs/testing-launch-quality.md`.

## Rust

- Unit tests live next to the code; pure domain tests in the domain crates.
- `make test-affected FAST_TEST_FILTER=<name>` for one test;
  `make test` for the workspace; CI also runs
  `cargo test -p oikonotes-app-kit -p oikonotes-app-shell -p oikonotes-app
  --no-default-features --features ssr-web --lib` (the unit tests live in the
  kit and shell crates; the root has none).

## Browser (Playwright)

| Config | Runs |
|---|---|
| `playwright.config.ts` | main release suite in `e2e/` (resets data first via `npm run test:e2e`) |
| `playwright.audit.config.ts` | debug/audit/UX-audit specs excluded from the main suite |
| `playwright.canary.config.ts` | `e2e-canary/` smoke against a deployed environment |
| `playwright.seed.config.ts`, `playwright.candidate-seed.config.ts` | seed data from `e2e-seed/` |

- Agents: `scripts/devx/e2e-env.sh up --build` gives the worktree its own
  desktop-mode server (port 3300–3399, data under `target/e2e-run/`);
  `e2e-env.sh test <args>` runs chromium with `--no-deps --max-failures=3`
  against it (single test: `e2e/<spec>.spec.ts:<line>`, reruns:
  `--last-failed`); `e2e-env.sh down` stops it. `status` says whether it is up.
  Run `up --build` again after each code change: it rebuilds and restarts. In
  Codex's sandbox the server dies with the command that started it, so run
  `up --build && test <args>; down` as one command.
- By hand: `make test-ui-focused E2E_SPEC=e2e/<spec>.spec.ts` against `make dev`
  (port 3000, needs a working `.env`).
- `test-results/`, `e2e-report/`, `e2e-results/` and snapshot folders are
  output; never commit them unless the change is an intended snapshot update.

## Contracts and other gates

- `npm run test:contract` (product contract in `quality/`) and
  `npm run test:contract:ipc` (Tauri IPC wiring).
- `make architecture-check`, `make pin-check`, `make docker-planner-check`.
- Desktop: `make e2e-desktop` (WebdriverIO, `e2e-desktop/`).
- Copepod connectivity: `make copepod-smoke` (`--full` flow needs test
  credentials).
