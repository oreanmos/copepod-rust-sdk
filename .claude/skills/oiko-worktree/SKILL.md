---
name: oiko-worktree
description: The worktree, commit and merge lifecycle for Oikonotes, Copepod and copepod-rust-sdk code changes - where worktrees live in each repo, per-worktree Cargo build dirs, commit conventions, integrating the base branch, merging back with --no-ff, pruning, and never touching other sessions' work. Load before creating, entering, merging or removing a worktree in any of the three repos, and when the base branch has moved or is dirty at merge time.
---

# Oiko Worktree Lifecycle

Implement in a worktree slot, merge back with `--no-ff`, then release the slot.
Claude, Codex and the owner work in these repos at the same time; most rules
below keep your work out of theirs.

## The repos

All three are siblings under `~/Development/` and share one layout. On a new
machine, clone them side by side and run
`bash .claude/skills/oiko-build/scripts/setup-machine.sh` once: it fetches the
shared agent state (build logs, Claude memory, Pi config) into
`~/Development/agent-state` and writes the machine's absolute paths into untracked
local settings. Pull that state repo before resuming a build on another
machine, and commit and push it when you stop.

| Repo | Base branch | Task worktrees | Notes |
|---|---|---|---|
| `oikonotes` | `main` | `oikonotes/.worktrees/slot-<n>` | no `Co-Authored-By` trailers (owner's rule); older sibling worktrees `../oikonotes-*` belong to earlier tasks, leave them |
| `copepod` | `main` | `copepod/.worktrees/slot-<n>` | `staging` is the carrier (Two machines) and deploys the staging cluster on push; the `dev` remote branch is never touched unless asked |
| `copepod-rust-sdk` | `main` | `copepod-rust-sdk/.worktrees/slot-<n>` | small standalone crate; no CI, no tags |

`.worktrees/` is gitignored in each. Branches: `feat/<scope>-<desc>`,
`fix/<scope>-<desc>`, `chore/<scope>-<desc>`, `refactor/<scope>-<desc>`; for a
spec slice, `<type>/<slug>-s<n>`. Commits: conventional
(`feat(scope): …`, `fix(scope): …`), one coherent step each.

## Create

Worktrees are a pool of slots per repo that keep their build directory between
tasks. Cargo reuses a build only at the same path, so reusing the directory is
what keeps third-party crates built. Measured 2026-10-06 on oikonotes
`make ci`: 882–1127 s cold (quiet machine), about 144 s on a reused slot when
`crates/app` is untouched, about 7–11 min for an edit inside `crates/app`
(653 s measured, 5 min of it the app crate's test target).

```bash
~/Development/agent-state/bin/wt take <repo> <slug> --branch <branch> [--base <rev>] --agent <claude-code|codex|opencode|pi>
~/Development/agent-state/bin/wt take <repo> <slug> --detach <rev> --agent <agent>     # reviewers
~/Development/agent-state/bin/wt list
```

`take` prints the slot's absolute path (`<repo>/.worktrees/slot-<n>`) on the
new branch, started from `<base>` (default `main`). With every slot held it
exits 3 and lists the holders: wait for a release or pick other work; never
take a slot another session holds, and never create a worktree by hand. Record
the base commit. A slot starts from committed `HEAD`; uncommitted changes in
the main checkout do not follow it. Worktrees made the old way
(`<repo>/.worktrees/<slug>`) finish the old way and are removed at merge.

A session started inside a harness-made worktree (`claude --worktree`, Codex)
uses that one; do not nest another.

Files that exist only in the main checkout (untracked `.env`, a spec committed
after you branched) are absent from the worktree: read them by absolute path or
with `git show <base>:<path>`. Oikonotes web mode needs `.env`; copy it into the
worktree only for runtime checks and never stage it.

## Build directory

Every repo builds with `CARGO_TARGET_DIR=target/local`. It is relative, so each
slot has its own `target/local`, which stays when the slot is released. Never
point two checkouts at one target dir: Cargo judges freshness by mtime and will
reuse the other checkout's artifacts, producing phantom errors and false
passes. A new slot is seeded with a copy-on-write copy of the main checkout's
`target/local` (1–2 s, no disk until it diverges); later builds are
incremental. Wrap builds and gates in `~/Development/agent-state/bin/heavy`: it
queues for the machine's two build slots, sets the Cargo job count and test
threads, and waits while the load average is above 1.5 × cores, so parallel
agents cannot push the machine into thrashing. In an oikonotes slot always run
`scripts/devx/e2e-env.sh up --build`, never bare `up`: the previous task's
server binary is still there. On take, a slot's `target/local` over 70 GB is
trimmed: first the workspace crates' own artifacts (third-party crates stay
built, so the next gate rebuilds only the workspace, about 10 min for
oikonotes), then files untouched for two days, and only then everything.
Oikonotes reaches the cap every two or three slices.

Evidence, logs and screenshots go under `~/Development/agent-state/<slug>/`. Never
in `/tmp` or a session scratchpad: those are RAM-backed on this machine.

## Commit and integrate

Stage explicit paths, never `git add -A` (Playwright output, `test-results/`,
screenshots and `.env` must not be committed). Before handing back, merge the
base into your branch inside the worktree and resolve conflicts there without
filtering the output. Re-run the pre-merge gate on the integrated tree when the
merge brought in code the gate covers; `oiko-implement` (Cadence) says when an
earlier green gate still stands.

## Merge and release

From the main checkout of that repo, with the slot committed and clean:

- **Uncommitted changes in files your merge touches:** stop and report them. Do
  not stash, commit, restage or discard another session's work, even
  temporarily.
- **Unrelated dirty or untracked files** can stay.

```bash
git -C <repo> merge --no-ff <branch> -m "<type>(<scope>): <summary>" -m "<evidence: key checks, spec slice>"
~/Development/agent-state/bin/wt release <repo>/.worktrees/slot-<n>
git -C <repo> branch -d <branch>
```

In that order: a branch checked out in a slot cannot be deleted. `release`
stops the slot's e2e server, refuses while uncommitted or untracked files
remain or the server is still up, detaches `HEAD`, removes ignored files
(`.env`, test output) and keeps `target/local`. Stop anything else you started
by its recorded PID. Never `pkill`, `killall` or kill by name or port; other
sessions' processes share the machine. If `release` refuses, inspect and
report; never delete tracked files to make it succeed. Release a slot as soon
as its branch merges or is abandoned: three slots per repo serve every session
on the machine.

After merging, push `main:staging` (next section) and report
`git -C <repo> status -sb`.

## Two machines

The owner works on a desktop and a laptop. Claude Code, Codex, OpenCode and Pi
sessions can run on both at once. Two things keep them apart:

- **Claims** in the state repo (`~/Development/agent-state/bin/claim`; see
  `oiko-build`, Before starting). They say who builds what.
- **`origin/staging`** in each of the three repos carries merged work between
  machines, and is the live channel: pushing copepod `staging` deploys the
  staging cluster, and pushing oikonotes `staging` builds the image and rolls
  it onto staging Oikonotes (`https://oikonotes.local.copepod.app`, see
  `docs/ops/release-verification.md`, Staging channel). The SDK's `staging`
  runs nothing. `origin/main` is production: pushing copepod `main` deploys
  prod, and pushing oikonotes `main` builds the release container.
  build.ulev.org pull-mirrors GitHub, so pushes go to `origin` only; when
  `FORGEJO_TOKEN` is set, trigger the mirror after the push:
  `curl -fsS -X POST -H "Authorization: token $FORGEJO_TOKEN"
  https://build.ulev.org/api/v1/repos/cesarli/<repo>/mirror-sync` (oikonotes
  and copepod).

| When | Do |
|---|---|
| Before claiming or starting a slice | `git -C <repo> fetch origin`. If `origin/staging` is ahead, `git -C <repo> merge --ff-only origin/staging` on `main` |
| `origin/next` has commits `origin/staging` lacks | `next` was the carrier until 2026-10-10 and a session on the old rules pushed it. Merge `origin/next` into `main` as in the row below, push `main:staging`, and tell that session to re-read this section |
| Local `main` and `origin/staging` both moved | `git -C <repo> merge --no-ff origin/staging -m "merge: staging from <machine>"`. Never rebase: it rewrites the `--no-ff` merges. Re-gate only if the merged code overlaps yours (Cadence in `oiko-implement`) |
| After every merge into `main` | `git -C <repo> push origin main:staging`, then the mirror-sync call when `FORGEJO_TOKEN` is set. If it is rejected, integrate `origin/staging` as in the row above, then push again |
| Release or deploy | Only when the owner asks: `git -C <repo> push origin staging:main`, from one machine at a time, after integrating `origin/staging`; for oikonotes then `scripts/deploy.sh --restart` |

Pushing is outward-facing. The owner has pre-authorized two pushes: `main:staging`
in these repos (it deploys only the staging environment, never production), and
the state repo. Every other push waits for the owner to ask, including `main`,
tags and feature branches. Never force-push `staging`. Every slice report names
the staging URL and the routes to open once
`curl -s https://oikonotes.local.copepod.app/__version` reports the merge sha with
`assets_ok:true`; until then it says staging is not updated (a green run with no
webhook secret rolls nothing out).
