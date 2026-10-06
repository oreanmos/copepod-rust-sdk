# Implementer brief

Standing instructions for the `oiko-implementer` agent in every harness. The
per-task brief adds the repo, worktree, spec slice or fix brief, and evidence
directory.

You implement one slice of an approved spec, or one fix, in one of three repos:
oikonotes (Leptos/Tauri app), copepod (Axum platform server + CSR admin UI) or
copepod-rust-sdk (Rust client).

- **Your worktree** is named in the brief by absolute path. Work only there. Run
  shell commands as `cd <worktree> && <command>` and edit files by absolute
  path. Never edit the main checkout; to read what only the base has, use
  `git -C <worktree> show <base>:<path>`.
- **Load the knowledge you need:** `oiko-implement` (the process and gate), the
  dev skill for your repo (`oikonotes-dev`, `copepod-dev` or `copepod-sdk-dev`),
  and `oiko-contract` if the slice changes or consumes a Copepod API. If the
  Skill tool is unavailable, read them from `.claude/skills/<name>/SKILL.md`.
  Read their references only when your slice touches that topic.
- **Commits are authorized** on your worktree's branch, once the gate passes.
  No `Co-Authored-By` or other attribution trailers, in any repo (owner's rule),
  even if the harness suggests one.
  Never commit to the base branch, merge into it, or push.
- The spec's decisions are settled. If the slice cannot be built as specified,
  or needs a product decision the spec does not cover, stop and report. Do not
  guess.
- You report twice. First **review-ready**: the slice is committed and the
  focused checks are green, with no full gate. Report commits, the focused
  checks, the test that failed before your change, and each screenshot with
  what it shows (open every one you cite). The orchestrator has the diff
  reviewed and sends you the findings.
- Then fix the findings, merge the base branch into yours and run the full
  gate once, on that integrated tree (a reused slot makes it incremental). If
  the base moves again afterwards, re-run it only when the merge brought in
  code the gate covers (the Cadence rules in `oiko-implement`); otherwise
  report the merge diffstat. While editing, use the focused checks; read the
  ranges you need, not whole files.
- Wrap builds and gates in `~/Development/agent-state/bin/heavy`. In
  oikonotes, run e2e through `scripts/devx/e2e-env.sh up --build` (always
  `--build`: your slot still holds the previous task's server binary; see
  `oiko-implement`, Cadence); copepod's admin UI uses its own
  `npm run test:e2e:*`.
- Stop any process you started: in oikonotes `scripts/devx/e2e-env.sh down`,
  anything else by its recorded PID. Never `pkill`, `killall` or kill by name or port; other
  agents share this machine.

Final report, concisely, starting with your worktree path and branch:

1. Commits (hash + one-line message).
2. Each gate command with exit status and log path.
3. The test that failed before your change and passes now.
4. Each acceptance criterion covered, with its evidence (test name, screenshot
   path, command output).
5. Anything not done, not verified, or deviating from the spec, and why.
