# Tests and e2e

Release methodology, fixtures and product contract:
`docs/testing-launch-quality.md`.

## Rust

- Unit tests live next to the code; pure domain tests in the domain crates.
- `make test-affected FAST_TEST_FILTER=<name>` for one test;
  `make test` for the workspace; CI also runs
  `cargo test -p oikonotes-app --no-default-features --features ssr-web --lib`.

## Browser (Playwright)

| Config | Runs |
|---|---|
| `playwright.config.ts` | main release suite in `e2e/` (resets data first via `npm run test:e2e`) |
| `playwright.audit.config.ts` | debug/audit/UX-audit specs excluded from the main suite |
| `playwright.canary.config.ts` | `e2e-canary/` smoke against a deployed environment |
| `playwright.seed.config.ts`, `playwright.candidate-seed.config.ts` | seed data from `e2e-seed/` |

- One spec during development:
  `make test-ui-focused E2E_SPEC=e2e/<spec>.spec.ts` (chromium, no deps).
- Needs a running app (`make dev`) with a working `.env`.
- `test-results/`, `e2e-report/`, `e2e-results/` and snapshot folders are
  output; never commit them unless the change is an intended snapshot update.

## Contracts and other gates

- `npm run test:contract` (product contract in `quality/`) and
  `npm run test:contract:ipc` (Tauri IPC wiring).
- `make architecture-check`, `make pin-check`, `make docker-planner-check`.
- Desktop: `make e2e-desktop` (WebdriverIO, `e2e-desktop/`).
- Copepod connectivity: `make copepod-smoke` (`--full` flow needs test
  credentials).
