---
name: oiko-loop
description: "Continue the Oikonotes / Copepod / copepod-rust-sdk / leptos-daisyui roadmap - interactively when the owner asks to carry on in any words ('continue', 'where were we', 'keep going', 'work on the roadmap'), brief where things stand, start the parallel workers, resume or pick the next item and build it visibly through oiko-build, and offer to plan with the owner when nothing is available; headless, one unattended iteration per worker session ending with a LOOP-RESULT line. Load it for any request to continue or work the roadmap."
effort: medium
---

# Oiko Loop

You are one orchestrator session started by `~/Development/agent-state/bin/oiko-loop` (or by the
owner). Several such sessions run at once on this machine. The owner is not watching: never wait for
an answer; record questions and work around them. Load `oiko-build` (and `oiko-plan` for planning
items) and follow them; this skill only adds how to pick work and how to stop.

## 0. Interactive: the owner asks to continue

Any request that means "carry on with the roadmap" ("continue", "let's keep going", "where were
we", "work on the roadmap", ...) in an interactive session makes you the **lead orchestrator**,
and the owner watches you work. Match the intent, not the words.

1. **Where we are**, first, in a short brief (about ten lines):
   - active builds: `claim list`, and for each this machine holds, the last entries of
     `~/Development/agent-state/<slug>/build-log.md` (what step, what is next);
   - the parallel workers: `~/Development/agent-state/bin/oiko-loop status` (alive sessions,
     last results, paused reason);
   - open owner questions (`owner_answers` / `/questions`), parked items in `loop/queue.md`;
   - what landed since the last session (`git log --merges` on `main` of each repo, staging
     `__version`), and the Mistral and Anthropic budget (`budget status`).
2. **Continue**: never start the parallel workers yourself; the owner starts them with
   `/loop start` (and they stop when the pi that started them closes). If they are not running,
   say so in one line and continue alone. Work an item with sections 1–4 below, visibly: resume
   one of your own stale claims first, else the next available item. After an item ends (done,
   parked or blocked), pick the next; do not stop at the "about six slices" boundary (pi compacts
   your context; the build log is your memory). One short line per step for the owner.
3. **Nothing available** (everything is done, claimed by a live worker, or waits on the owner):
   say so, list the open owner questions with their recommendations, and offer to plan the next
   thing together: name the top two or three candidates from the launch schedule that have no
   approved spec, with why each matters for launch, and run `oiko-plan` as a dialog with the
   owner on the one they pick. Interactive sessions do not draft specs unattended (section 2,
   item 6 is for headless workers).
4. Owner questions raised while working: ask directly if it blocks the item you are on and the
   answer is quick; otherwise record them (`owner_question`) and keep working on everything that
   does not depend on them.
5. Stop when the owner says so or the Mistral budget is exhausted. Stopping the workers is the
   owner's call too (`/loop stop`, or closing the pi that started them).

## 1. Start

1. `cd ~/Development/oikonotes`. Sync per `oiko-worktree` (Two machines): fetch every repo you may
   touch and bring `main` (leptos-daisy: `master`) up to `origin/staging` (`origin/master`).
2. `~/Development/agent-state/bin/budget check mistral`: exit 3 → print
   `LOOP-RESULT: blocked - mistral budget exhausted` and stop.
3. Read answers: Pi `owner_answers`; elsewhere the `answer` lines of
   `~/Development/agent-state/loop/owner-questions.jsonl`. An answered question unparks its item.
4. Read `~/Development/agent-state/owner-notes.md` and `claim list`.
   Work can be cut off mid-step (workers stop when the owner's pi closes). Before resuming an item,
   check its worktree and the main checkouts for an interrupted step: `git status`, an unfinished
   merge or rebase (abort it unless the build log says it completed), uncommitted edits (keep an
   implementer's work in its worktree; edits left on a main checkout by a killed session go to a
   patch under `~/Development/agent-state/loop/interrupted/` and the file is restored), and patches
   already saved there for your item.
5. Read the standing operational rules earlier sessions learned:
   `~/Development/agent-state/_home/claude-memory/oikonotes/MEMORY.md` (an index; open the notes
   that touch your item: e2e lock, heavy slots, staging pushes cancel builds, wasm budget raises,
   copepod pushes deploy prod, browser tests before merge). They bind you like this skill.

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
