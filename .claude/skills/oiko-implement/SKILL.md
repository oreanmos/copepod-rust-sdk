---
name: oiko-implement
description: Implement one Oikonotes, Copepod or copepod-rust-sdk change whose right behaviour is already clear - one slice of an approved spec, or a fix, test, refactor, dependency bump or config change - in a worktree, proven with a failing-first test and the repo's gate. Run by the oiko-implementer subagent that oiko-build spawns; a main session with subagents loads oiko-build instead. New features or anything needing product decisions go to oiko-plan first.
effort: medium
---

# Oiko Implementation

In a harness with subagents, this skill is run by `oiko-implementer`, one slice
at a time, briefed by `oiko-build`. If you are the main session there, load
`oiko-build` and delegate; implement here yourself only when the harness has no
subagents or the owner asked for it in this session (then create the worktree
yourself per `oiko-worktree`).

Switch to `oiko-plan` (say why first) when reproducing the problem shows it
needs a product decision, a change to how something looks or works, or more than
one slice.

## Before editing

1. Work in the task worktree; load the repo's dev skill (`oikonotes-dev`,
   `copepod-dev`, `copepod-sdk-dev`) and, for API work, `oiko-contract`.
2. Locate the relevant code with scoped searches; read the ranges you need. Do
   not read broad docs unless you are changing that subsystem's contract.
3. Reproduce: write or find the test that shows the current behaviour, and watch
   it fail for the right reason before changing code.
4. State in two or three lines: scope, files, exclusions, and how you will
   verify.

## Implementation

- Make the smallest production-complete change that meets the acceptance
  criteria: no placeholders, no speculative abstraction, no unrelated cleanup.
- Keep the layering the repo's dev skill describes (domain → persistence → API
  or UI). Validate external input at boundaries.
- Add or update focused tests for changed behaviour and every new public API.
  Doc-comment public items; comment only intent the code cannot show.
- Respect file-size caps: oikonotes ≤300 lines target / 500 hard; copepod 500
  hard. Split by concern, never by arbitrary chunks, and never game a check.
- If your change invalidates existing state (a schema, a stored record shape, a
  setting), ship the migration in the same slice.
- Never commit secrets, `.env` files, tokens, or a `Cargo.lock` produced by a
  local `[patch]` override.

## Gate

Run each check as its own command with output to a log in the evidence
directory and the status recorded:

```bash
cd <worktree> && <command> > <evidence>/<name>.log 2>&1; echo "exit=$?"
```

Never pipe a check into `tail`/`grep`/`head` (you would read the filter's
status). Start long checks with `run_in_background` and end your turn; the
notification resumes you. Do not poll or `sleep`. Every repo uses
`CARGO_TARGET_DIR=target/local` (relative, so each worktree gets its own; never
`/tmp`).

**oikonotes** (Make targets already set the target dir)

| When | Commands |
|---|---|
| While editing | `make check-affected`, or the surface you touched: `make app-ssr` (server, server fns, shared UI), `make app-hydrate` (islands, client), `make app-csr` + `make tauri-check` (desktop-visible UI/API); `make test-affected FAST_TEST_FILTER=<name>` |
| Each commit | `make fmt-check`, the surface checks above, focused tests |
| Before reporting (integrated tree) | `make ci` (preflight, clippy, workspace tests, all app surfaces, tauri) · `scripts/check-state-pool-boundary.sh` · `scripts/check-no-committed-credentials.sh` · UI changes: `make test-ui-focused E2E_SPEC=e2e/<spec>.spec.ts` plus `e2e/mobile-responsive.spec.ts` for layout |

**copepod** (prefix cargo with `CARGO_TARGET_DIR=target/local`)

| When | Commands |
|---|---|
| While editing | `cargo check -p <crate>`, `cargo test -p <crate> <filter>` |
| Each commit | `cargo fmt --all -- --check`, checks/tests of touched crates |
| Before reporting | `cargo clippy --locked --workspace --exclude copepod-ui --all-targets -- -D warnings` · `cargo test --locked --workspace --exclude copepod-ui` · routes or API docs changed: `python3 scripts/generate_openapi_surface.py --check` · admin UI changed: `cargo check -p copepod-ui`, `npm run build:ui:css`, `scripts/check-ui-css-versions.sh`, the relevant `npm run test:e2e:*` · deploy/release-impacting: `scripts/validate-p0-release-gates.sh` |

**copepod-rust-sdk** (standalone crate, no toolchain file)

| When | Commands |
|---|---|
| Each commit | `cargo fmt -- --check`, `cargo test --test <file>` |
| Before reporting | `cargo clippy --all-targets -- -D warnings` · `cargo test` |

A check that is equally red on the base commit is pre-existing: name it with the
base-commit evidence and leave it alone. Reuse a passing check only when nothing
it covers has changed since. Pure docs or agent-instruction changes need only
`git diff --check` and valid links.

## Finish

As an implementer, stop at your report (format in the standing brief); the
orchestrator merges. Otherwise merge per `oiko-worktree`. Every verification
claim names its command and exit status or its artifact path. Say plainly what
you did not exercise.
