---
name: oiko-loop
description: One unattended iteration of the Oikonotes / Copepod / copepod-rust-sdk / leptos-daisyui roadmap loop - pick the next unclaimed work item from the loop queue, owner notes and the launch schedule, claim it, build it through oiko-build (or draft its spec through oiko-plan), park what waits on the owner, and end with a LOOP-RESULT line. Run by the oiko-loop runner's headless sessions (Pi or Vibe); load it when asked to "run one iteration of the loop" or to work the roadmap without the owner.
effort: medium
---

# Oiko Loop

You are one orchestrator session started by `~/Development/agent-state/bin/oiko-loop` (or by the
owner). Several such sessions run at once on this machine. The owner is not watching: never wait for
an answer; record questions and work around them. Load `oiko-build` (and `oiko-plan` for planning
items) and follow them; this skill only adds how to pick work and how to stop.

## 1. Start

1. `cd ~/Development/oikonotes`. Sync per `oiko-worktree` (Two machines): fetch every repo you may
   touch and bring `main` (leptos-daisy: `master`) up to `origin/staging` (`origin/master`).
2. `~/Development/agent-state/bin/budget check mistral`: exit 3 → print
   `LOOP-RESULT: blocked - mistral budget exhausted` and stop.
3. Read answers: Pi `owner_answers`; elsewhere the `answer` lines of
   `~/Development/agent-state/loop/owner-questions.jsonl`. An answered question unparks its item.
4. Read `~/Development/agent-state/owner-notes.md` and `claim list`.

## 2. Pick one item, first match wins

An item is **available** when: its slug is not claimed by anyone else (a claim of yours is
available only if `~/Development/agent-state/loop/active/<slug>` is missing or names a dead PID),
nothing it depends on is unmerged or claimed, its areas do not overlap an active claim, and it is
not parked on an open owner question.

1. **Resume your own:** a claim with agent `pi` (or `vibe`) on this machine whose active file is
   stale: resume it from its build log.
2. **Takeovers the owner granted** in `~/Development/agent-state/loop/queue.md` (section
   Takeovers): resume from the build log; `claim take` with `--override "owner takeover
   2026-11-16"` only where the queue says so.
3. **Owner notes** lines without `→` inside no active build's area: a one-slice fix build.
4. **The queue** (`loop/queue.md`), top to bottom.
5. **The launch schedule's current list** (`docs/plans/2026-10-13-launch-schedule.md`, the latest
   amendments and their "Remaining before G1" table), lane order, items whose spec is approved.
6. **Planning:** the first item there with no decision-complete spec. Run `oiko-plan`
   unattended: ground it in the code, write the spec as a draft with every open product decision
   recorded as an owner question (recommendation first, options), mark the spec
   `Status: draft — waiting on owner questions <ids>`, commit it on `main` (docs only), push
   `main:staging`. Never build an unapproved spec.

Nothing available → print `LOOP-RESULT: idle - nothing available` and stop.

## 3. Work it

- `claim take <slug> ... --agent pi` (Vibe: `--agent vibe`); write your PID (Pi: the parent of your
  bash tool, `echo $PPID`) to `~/Development/agent-state/loop/active/<slug>`.
- Build with `oiko-build`: Red → Green → code and security review → gate → merge →
  `main:staging`, then acceptance, the Opus go/no-go and the release to `main` in repo order, as
  that skill says. Owner decisions the spec does not settle: record the question, park only the
  part that depends on it, continue with the rest.
- Keep the build log current after every step: the next session resumes from it, not from you.
- **Stop at a step boundary** (log, claim and state repo updated) when any of these holds: the
  spec is accepted and released or waiting on its gate; everything left is parked; your context
  has been compacted once or you have merged about six slices; the Mistral budget is exhausted.

## 4. End

1. Remove `loop/active/<slug>`. Accepted (or abandoned): `claim release <slug>`. More to do, or
   parked: `claim update <slug> "<state>; next: <step>"` and keep the claim, unless everything
   left waits on the owner: then release it and add a `Parked:` line under the item in
   `loop/queue.md` with the question ids (take `flock ~/Development/agent-state/loop/queue.lock`
   for the edit).
2. Commit and push the state repo.
3. Last line of your output, exactly one of:
   `LOOP-RESULT: done <slug> <summary>` (accepted or released),
   `LOOP-RESULT: progress <slug> <summary>` (more to do, no blocker),
   `LOOP-RESULT: parked <slug> <question ids>`,
   `LOOP-RESULT: blocked <slug> <reason>`,
   `LOOP-RESULT: idle - <reason>`.
