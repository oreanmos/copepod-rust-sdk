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
| `copepod` | `main` | `copepod/.worktrees/slot-<n>` | also has `staging` and `dev` remote branches; never merge into them unless asked |
| `copepod-rust-sdk` | `main` | `copepod-rust-sdk/.worktrees/slot-<n>` | small standalone crate; no CI, no tags |

`.worktrees/` is gitignored in each. Branches: `feat/<scope>-<desc>`,
`fix/<scope>-<desc>`, `chore/<scope>-<desc>`, `refactor/<scope>-<desc>`; for a
spec slice, `<type>/<slug>-s<n>`. Commits: conventional
(`feat(scope): …`, `fix(scope): …`), one coherent step each.

## Create

Worktrees are a pool of slots per repo that keep their build directory between
tasks. Cargo reuses a build only at the same path, so reusing the directory is
what turns a 20-minute cold gate into a 2-minute warm one (measured 2026-10-06:
oikonotes `make ci` 1127 s in a fresh worktree, 144 s on a reused slot).

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
server binary is still there. `wt` deletes a slot's `target/local` over 70 GB
on take.

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

After merging, push `main:next` (next section) and report
`git -C <repo> status -sb`.

## Two machines

The owner works on a desktop and a laptop. Claude Code, Codex, OpenCode and Pi
sessions can run on both at once. Two things keep them apart:

- **Claims** in the state repo (`~/Development/agent-state/bin/claim`; see
  `oiko-build`, Before starting). They say who builds what.
- **`origin/next`** in each of the three repos carries merged work between
  machines. `origin/main` is what is deployed: pushing copepod `main` deploys
  prod, and pushing oikonotes `main` builds the release container. No workflow
  runs on `next`.

| When | Do |
|---|---|
| Before claiming or starting a slice | `git -C <repo> fetch origin`. If `origin/next` is ahead, `git -C <repo> merge --ff-only origin/next` on `main` |
| Local `main` and `origin/next` both moved | `git -C <repo> merge --no-ff origin/next -m "merge: next from <machine>"`. Never rebase: it rewrites the `--no-ff` merges. Re-gate only if the merged code overlaps yours (Cadence in `oiko-implement`) |
| After every merge into `main` | `git -C <repo> push origin main:next`. If it is rejected, integrate `origin/next` as in the row above, then push again |
| Release or deploy | Only when the owner asks: `git -C <repo> push origin next:main`, from one machine at a time, after integrating `origin/next` |

Pushing is outward-facing. The owner has pre-authorized two pushes: `main:next` in
these repos, and the state repo, both of which deploy nothing. Every other push
waits for the owner to ask, including `main`, `staging`, tags and feature branches.
Never force-push `next`.
