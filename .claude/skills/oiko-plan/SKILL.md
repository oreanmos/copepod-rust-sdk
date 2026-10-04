---
name: oiko-plan
description: Turn an Oikonotes or Copepod request into an agreed, decision-complete spec through a short dialog with the owner, grounded in the real code of all three repos (oikonotes, copepod, copepod-rust-sdk) - or investigate without changing code. Use for every new feature, screen, data domain or platform capability ("add X to oikonotes", "copepod should support Y", "I want Z in the app"), any change that may cross the server → SDK → app boundary, fixes that need product decisions, "plan", "spec", "figure out what's going on", and investigations. The spec it writes is what oiko-build executes.
effort: high
---

# Oiko Planning

Follow the repo's AGENTS.md for scope, reading, waiting and evidence. Do not
modify production code.

## Roles

The owner decides what the feature does, how it feels, what matters most, and
which trade-offs users live with. You are the expert on Rust, Leptos, Axum,
Tauri, API design and this codebase: make those calls, record each in the spec
with a one-line plain-language reason, and say what the owner will see as a
result. Never ask the owner to pick a data structure, endpoint shape, crate or
store layout. When a technical choice changes something visible (offline
support, latency, what happens to existing data), ask about the outcome instead.

## 1. Ground yourself

Delegate the gathering and keep your context for decisions.

- "Where is X / how does Y work" questions: `Explore` subagents with
  `model: "haiku"`, in parallel, one per repo or question. Ask for `file:line`
  answers, not file dumps.
- Behaviour in the running app (a screen, a flow, a bug): `oiko-scout` in the
  background, briefed per
  [../oiko-build/references/scout.md](../oiko-build/references/scout.md).
- Read yourself only the code the decisions hinge on.

## 2. Decide which repos the change touches

This decides the slice list and most of the cost, so settle it before asking the
owner anything.

- **Oikonotes only** — most product features. Copepod's generic collections,
  records, files and realtime already cover ordinary persistence: a new data
  domain is a new store trait plus a `CopepodXStore` over a collection, not a
  server change. Check `crates/app/src/ssr/copepod_store/` for a similar domain
  first.
- **Copepod + SDK + Oikonotes** — only when the platform lacks a capability:
  a new endpoint, auth flow, billing/email/ticket behaviour, server-side
  validation, or a performance need records cannot meet. Load `oiko-contract`
  and classify the change as additive or breaking.
- **Copepod only / SDK only** — platform work with no app change yet.

## 3. Confirm the problem and ask

Restate in a few sentences what the owner wants, what you found, and what
"done" looks like. Then ask with `AskUserQuestion` (plain numbered questions in
other harnesses): at most four per round, recommended option first, each option
described by what the user will see. Cover only what evidence cannot settle:
the user-visible goal, scope in and out, priorities, web vs desktop behaviour,
offline expectations, what happens to data users already have, and how the
owner will judge it works. Usually one or two rounds. When you have made a
technical decision the owner might care about, state it instead of asking.

If the feature adds or changes UI and the owner gave no design, describe the
screen in terms of existing pages and `leptos-daisyui` components and let them
pick between at most three concrete layouts (ASCII previews in the question).

## 4. Design and slice

Choose the approach; name rejected alternatives in one line each. Split into
slices that each merge on their own, in dependency order:

1. Copepod server (handler, route, models, API reference, OpenAPI surface).
2. copepod-rust-sdk (client method, models, wiremock test).
3. Oikonotes (pin bump, bridge, store trait + Copepod and local impls, server
   functions, UI).

Each slice names its repo. Keep a slice to one repo. Check file headroom for
files a slice grows: oikonotes targets ≤300 lines (500 hard), copepod has a 500
line cap with an inventory in `docs/production-hardening-open-items.md`.

## 5. Write the spec

Write `docs/plans/YYYY-MM-DD-<slug>.md` in the repo that owns the user-visible
outcome (oikonotes for product features, copepod for platform-only work), using
[references/spec-template.md](references/spec-template.md). Write it so an
implementer on a smaller model can build each slice from the spec alone: files
and symbols, endpoint paths and shapes, decisions taken, tests to add, and the
evidence each acceptance criterion needs.

Show the owner a short summary (behaviour, owner decisions, expert decisions,
slices by repo, how acceptance is checked) and ask with "Approve and build" as
the recommended option. Revise until approved.

## 6. Record approval and build

Set the status to `approved YYYY-MM-DD`, commit only the spec on the base branch
(`git add docs/plans/<file>` then `git commit -m "docs(plans): <slug> spec"`),
and push `main:next` so the other machine sees it (`oiko-worktree`, Two machines).
Load `oiko-build` and start the first slice in the same turn, unless the owner
said to hold; its claim step runs first. When you propose what to build next,
check `~/Development/agent-state/bin/claim list` and leave out claimed work.

## Investigations

When the task is to understand, not build: deliver findings, evidence,
uncertainty and recommendations. Say which findings you reproduced and which
rest on reading. Offer to turn recommendations into a spec. Do not force the
template onto a question.
