# copepod-rust-sdk Agent Contract

`copepod-sdk` is the typed Rust client for the Copepod platform API. The server
is `../copepod`; the main consumer is `../oikonotes`, which pins this crate by
full commit SHA from GitHub. Changes here are usually one slice of a
cross-repo feature: server → SDK → Oikonotes.

## Task routing

| Request | Skill |
|---|---|
| A new capability or anything with open product decisions; investigations | `oiko-plan` → spec → `oiko-build` |
| An approved spec, "go", "implement this plan" | `oiko-build` |
| Expose an existing server endpoint, fix a client bug, add tests | `oiko-build` as a one-slice build |
| Review, check a branch | `oiko-reviewer` subagent per `oiko-review` |
| Docs, agent instructions only | edit directly on `main`; after changing the shared kit run `bash .claude/skills/oiko-build/scripts/sync-agent-kit.sh` |

Reference skills: `copepod-sdk-dev` (this repo), `oiko-contract` (every API
change and the Oikonotes pin bump), `copepod-dev`, `oikonotes-dev`,
`oiko-worktree`. Subagents: `oiko-implementer`, `oiko-reviewer`, `oiko-scout`
(see `oiko-build`). Implement in the main session only when the harness has no
subagents or the owner asks.

Parallel work: Claude Code, Codex, OpenCode and Pi sessions run on two machines
at once. Before starting a build, sync from `origin/next` and claim it with
`~/.cache/oiko-agents/bin/claim`; push `main:next` after each merge (pre-authorized,
deploys nothing). Never take claimed work. One implementer per machine. See
`oiko-build`, `oiko-worktree` (Two machines) and `oikonotes/docs/agents.md`.

## Invariants

- The SDK mirrors the server's documented API (`copepod/docs/api-integration-reference.md`);
  it never invents endpoints or shapes.
- Validate arguments before any network call; map every non-2xx response to
  `CopepodError::Api` with the server's code preserved.
- Bearer tokens and API keys are never sent together.
- Additive changes only, unless a coordinated breaking build says otherwise.
- Every public method has a doc comment and a wiremock test.
- Nothing reaches `origin` without the owner's say-so; Oikonotes can only pin
  pushed commits.

## Execution and verification

Smallest complete change; locate with scoped searches. Gate before merge:
`cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, each with its exit status recorded and never piped into a
filter. Report changed files, checks with exit status, the rev Oikonotes should
pin, and whether a push is still needed.
