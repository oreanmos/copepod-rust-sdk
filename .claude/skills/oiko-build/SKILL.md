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
| A clear fix in one repo | one-slice build from a fix brief; reviewer if users will see the change or it touches auth, user data, billing or isolation |
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
- Build log: `~/Development/agent-state/<slug>/build-log.md` — each slice's repo,
  status, worktree, branch, commits, evidence paths, decisions and next step.
  After compaction or a pause, resume from the log, not from memory.
- One orchestrator session per spec. When the spec is accepted, or the session
  has run about a day, finish with the log up to date and continue in a fresh
  session: a long session re-reads its whole context on every turn, and the
  three longest of 2026-09-26..10-03 used about a third of all tokens.
- **Sync and claim.** Several machines and harnesses build at once. In each
  repo the work touches, `git fetch origin` and bring `main` up to
  `origin/next` (`oiko-worktree`, Two machines). Then run
  `~/Development/agent-state/bin/claim list`. Do not start work if its slug is
  claimed, if a step it depends on is claimed or is not yet on `next`, or if its
  areas overlap an active claim. In those cases, pick other work or ask the
  owner. Otherwise run
  `claim take <slug> --step "<task-order step / spec>" --repos <r1,r2>
  --areas "<dirs>" --agent <claude-code|codex|opencode|pi>`. A refused `take`
  means another session has the work. Resuming your own claimed build needs no
  new claim. If `git worktree list` shows another session's branch for your
  next slice, stop and ask the owner.
- A notification that an agent is still running needs a one-line reply and no
  tool calls.

## Each slice, in order

Up to three builds run per machine: `claim take` refuses a fourth, and any new
build under 100 GB free disk (`--override "<reason>"` only on the owner's word).
Heavy commands queue in `~/Development/agent-state/bin/heavy` (two slots), so
parallel agents overlap their thinking, not their compiles. Within one build,
run one implementer at a time unless slices are independent and share no files.
Read-only lookups can run alongside.

1. **Worktree.** Make sure the slice's repo base branch has what the slice needs
   committed. Take a slot per `oiko-worktree`:
   `~/Development/agent-state/bin/wt take <repo> <slug> --branch <type>/<slug>-s<n> --agent <agent>`,
   and brief the implementer with the path it prints. All slots held: finish
   or release what you hold, or wait; do not create a worktree by hand.
2. **Delegate** to `oiko-implementer` in the background:

   ```text
   Agent(subagent_type: "oiko-implementer", description: "<slug> S<n>",
     prompt: "Repo: <repo abs path>. Worktree: <abs path>, branch <branch>, base <base>.
              Spec: <abs spec path>, slice S<n> (<name>).     <- or: Fix brief: <build-log path>
              Evidence dir: ~/Development/agent-state/<slug>/s<n>/.
              Acceptance criteria covered: #<k>, #<m>.
              Settled during the build: <decisions from earlier slices, SDK rev, or 'none'>.
              Standing brief: .claude/skills/oiko-build/references/implementer.md")
   ```

   End your turn; the completion notification resumes you.
3. **Review before the gate.** The implementer's first report is
   *review-ready*: committed, focused checks green, no full gate yet. Read
   `git -C <worktree> diff <base>...HEAD` against the slice goal, and its
   `--stat` file count (build output such as `crates/target/` must never be
   committed); confirm a test that failed before the change; for UI, open every
   screenshot. Then spawn `oiko-reviewer` for a branch review of that diff
   (`oiko-review`, Branch review: read-only, nothing heavier than a focused
   test), with the spec path, the slice and the worktree path. `SendMessage`
   its findings and your own to the same implementer (it keeps its context and
   its warm slot), which fixes them, integrates the base and runs the full
   gate once. Reviewers found every High defect of 2026-09-26..10-03 and the
   gates found none, and the slice reviews of 2026-10-06 sent fixes back on
   five slices out of five, so the gate runs on reviewed code, not before it.
4. **Check the gated report before merging** — never merge on a summary: read
   the diff since the review; open the gate logs and map every `.status` to its
   check by name; for API slices, confirm the contract artifacts the slice owes
   (see `oiko-contract`). Do not re-run the gate, and do not ask a reviewer to,
   when the base moved only by commits outside what the gate covers (the
   Cadence rules in `oiko-implement`); re-gate only when the evidence is
   missing, red, or predates an overlapping change.
5. **Gaps:** `SendMessage` the implementer with specifics. After two rounds
   without resolution, log a blocker and brief a fresh implementer with what
   the first learned.
6. **Merge** per `oiko-worktree` (`--no-ff`, evidence in the message), `wt
   release` the slot, delete the branch, push `main:next`, run `claim update <slug> "S<n> merged"` (it pushes the state
   repo), update the log, and go straight to the next slice.

## Cross-repo handoffs

- **SDK → Oikonotes needs the rev on GitHub.** Oikonotes pins `copepod-sdk`
  by git rev, so the SDK commit must be on `origin` before the Oikonotes slice
  can commit its pin bump. The push of SDK `main:next` after its merge (step 6)
  puts it there, which is pre-authorized. Pin that rev. Pushing SDK `main` stays
  the owner's call, as for every repo.
- **Deploy order.** An Oikonotes release that uses a new endpoint needs the
  Copepod server deployed first. Record this in the finish report; do not
  deploy unless asked.

## Acceptance

After the last slice of a spec, spawn `oiko-reviewer` against the merged base
branches and the spec. The slices' code was already read, so acceptance spends
its time exercising the merged feature as a user would. All defects of one
review round go into one fix slice per repo, through the same loop; review
again. Done when the reviewer accepts or the owner accepts a listed
exception. For specs with layout changes, the reviewer runs the full
`e2e/mobile-responsive.spec.ts` through `scripts/devx/e2e-env.sh` in its review
worktree; slices ran only their touched routes. For a fix, the step-3 review and your step-4 check are the acceptance unless users will see
the change or it touches auth, user data, billing or isolation. Do not skip
acceptance for UI-only specs: the 2026-10 media/records privacy exposure shipped
from a UI build that had no reviewer, and every High finding of that week came
from a reviewer, not from a gate.

## The owner

Ask only when a decision changes user-visible behaviour beyond what the spec
settles, when the spec is wrong, or before any push or deploy. Use
`AskUserQuestion`, recommendation first. Technical choices stay yours; log them.

## Finish

When the build is accepted, abandoned or handed back to the owner, run `claim release
<slug>`. Then commit and push the state repo, so the build log reaches the other machine.

Report, briefly: each slice with repo, merge commit and key evidence; the
acceptance verdict (or the fix evidence); owner decisions taken; deferred items;
pushes and deploys still needed, in order; cleanup status of worktrees.
