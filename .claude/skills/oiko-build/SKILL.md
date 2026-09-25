---
name: oiko-build
description: Orchestrate every Oikonotes / Copepod / copepod-rust-sdk code change to completion - an approved spec slice by slice across the three repos, or a fix as a single slice - through implementer subagents on a smaller model, checked and merged in dependency order (server → SDK → app), then accepted by a reviewer. Use when a spec is approved ("build <slug>", "go", "approve and build", "implement this plan"), and for every fix, bug, regression, test, refactor, dependency bump or config change in a harness that has subagents.
effort: medium
---

# Oiko Build

You orchestrate and judge. You do not write a slice's code or run its gate
yourself: that is the implementer's job, on a smaller model. Keeping your context
for judgement is what lets a multi-repo spec run to the end, and keeping code
work off the orchestrator's model is what keeps a build affordable. Follow the
repo's AGENTS.md. The owner has granted autonomy for the build: work through
every slice without checking in; stop only for a blocker, a product decision the
spec does not cover, or a push.

## Size the work first

| Work | Path |
|---|---|
| Docs, specs, agent instructions only | edit directly on the base branch; no subagents, no Cargo gate |
| A clear fix in one repo | one-slice build from a fix brief; reviewer only if users will see the change |
| An approved spec | slices in order, then `oiko-reviewer` acceptance |
| Anything with open product decisions or several repos and no spec | `oiko-plan` first |

## Roles and models

| Role | Claude Code | Codex | Does |
|---|---|---|---|
| Orchestrator (you) | the session model | `gpt-6-sol` | briefs, checks evidence, merges, keeps the log, talks to the owner |
| `oiko-implementer` | Sonnet, medium | `gpt-6-luna` | one slice or fix in its worktree, per `oiko-implement`; never merges |
| `oiko-reviewer` | Opus, high | `gpt-6-luna` | read-only acceptance, per `oiko-review` |
| `oiko-scout` | Sonnet, medium | `gpt-6-luna` | read-only runtime observation and code questions |
| Lookups | `Explore`, `model: "haiku"` | default subagent | "where is X" questions, in parallel |

Agent definitions set model and effort (`.claude/agents/`, `.codex/agents/`,
`.opencode/agents/`): pass no `model` override. Brief with paths, never pasted
content. Do not fork your own context into an implementer: a fork keeps your
model, which is the cost this split avoids. Where the harness has no subagents,
play each role in turn with the same briefs and checks.

## Before starting

- **Spec build:** the spec is approved and has no open questions; otherwise
  `oiko-plan`. **An existing plan the owner asks you to implement:** the request
  is the approval; read it and what has merged for it, slice the rest, settle
  technical gaps yourself, ask about open product decisions in one round.
- **Fix:** write a fix brief at the top of the build log: the problem in the
  owner's words, how to reproduce it, expected behaviour, acceptance criteria,
  repo and surfaces touched.
- Build log: `~/.cache/oiko-agents/<slug>/build-log.md` — each slice's repo,
  status, worktree, branch, commits, evidence paths, decisions and next step.
  After compaction or a pause, resume from the log, not from memory.

## Each slice, in order

One implementer at a time: builds of these workspaces are heavy and share the
machine. Read-only lookups can run alongside.

1. **Worktree.** Make sure the slice's repo base branch has what the slice needs
   committed. Create the worktree per `oiko-worktree`:
   `git -C <repo> worktree add -b <type>/<slug>-s<n> <repo>/.worktrees/<slug>-s<n> <base>`.
2. **Delegate** to `oiko-implementer` in the background:

   ```text
   Agent(subagent_type: "oiko-implementer", description: "<slug> S<n>",
     prompt: "Repo: <repo abs path>. Worktree: <abs path>, branch <branch>, base <base>.
              Spec: <abs spec path>, slice S<n> (<name>).     <- or: Fix brief: <build-log path>
              Evidence dir: ~/.cache/oiko-agents/<slug>/s<n>/.
              Acceptance criteria covered: #<k>, #<m>.
              Settled during the build: <decisions from earlier slices, SDK rev, or 'none'>.
              Standing brief: .claude/skills/oiko-build/references/implementer.md")
   ```

   End your turn; the completion notification resumes you.
3. **Check the report yourself before merging** — never merge on a summary:
   read `git -C <worktree> diff <base>...HEAD` against the slice goal; open the
   gate logs and confirm each exit status; confirm a test that failed before the
   change; for UI, open the screenshots; for API slices, confirm the contract
   artifacts the slice owes (see `oiko-contract`).
4. **Gaps:** `SendMessage` the implementer with specifics (it keeps its
   context). After two rounds without resolution, log a blocker and brief a
   fresh implementer with what the first learned.
5. **Merge** per `oiko-worktree` (`--no-ff`, evidence in the message), prune,
   update the log, go straight to the next slice.

## Cross-repo handoffs

- **SDK → Oikonotes needs a push.** Oikonotes pins `copepod-sdk` by git rev on
  GitHub, so the SDK commit must be on `origin` before the Oikonotes slice can
  commit its pin bump. After the SDK slice merges, ask the owner once:
  "Push copepod-rust-sdk main (<rev>) so Oikonotes can pin it?" If they hold,
  the Oikonotes implementer may verify against the local SDK with the cargo
  patch override in `oiko-contract`, but the pin bump waits; log it as blocked.
- **Deploy order.** An Oikonotes release that uses a new endpoint needs the
  Copepod server deployed first. Record this in the finish report; do not
  deploy unless asked.

## Acceptance

After the last slice of a spec, spawn `oiko-reviewer` against the merged base
branches and the spec. Each defect becomes a fix slice through the same loop;
review again. Done when the reviewer accepts or the owner accepts a listed
exception. For a fix, your step-3 check is the acceptance unless users will see
the change.

## The owner

Ask only when a decision changes user-visible behaviour beyond what the spec
settles, when the spec is wrong, or before any push or deploy. Use
`AskUserQuestion`, recommendation first. Technical choices stay yours; log them.

## Finish

Report, briefly: each slice with repo, merge commit and key evidence; the
acceptance verdict (or the fix evidence); owner decisions taken; deferred items;
pushes and deploys still needed, in order; cleanup status of worktrees.
