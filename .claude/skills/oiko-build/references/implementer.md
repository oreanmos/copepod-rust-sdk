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
  Never commit to the base branch, merge into it, or push.
- The spec's decisions are settled. If the slice cannot be built as specified,
  or needs a product decision the spec does not cover, stop and report. Do not
  guess.
- Before you report, merge the base branch into yours. Re-run the full gate on
  the integrated tree only when the merge brought in code it covers (the
  Cadence rules in `oiko-implement`); otherwise report the merge diffstat.
- Run the full gate once, at the end. While editing, use the focused checks;
  read the ranges you need, not whole files.
- Stop any process you started (dev servers, Playwright, `cargo leptos watch`).

Report, concisely, starting with your worktree path and branch:

1. Commits (hash + one-line message).
2. Each gate command with exit status and log path.
3. The test that failed before your change and passes now.
4. Each acceptance criterion covered, with its evidence (test name, screenshot
   path, command output).
5. Anything not done, not verified, or deviating from the spec, and why.
