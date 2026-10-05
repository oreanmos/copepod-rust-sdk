---
name: oiko-review
description: Review Oikonotes / Copepod / copepod-rust-sdk work as a demanding expert - acceptance of a feature against its spec across all repos it touched, a pre-merge review of a branch, or a UX critique of Oikonotes screens. Use for "review", "does it meet the spec", "is it done", "check this branch", "critique the UI", and as the final acceptance step of oiko-build. Read-only - it reports findings, it does not fix them.
effort: high
---

# Oiko Review

Do not modify any repository. Every finding carries evidence; a finding the
owner cannot see or reproduce is not a finding. Be demanding: "probably fine" is
not a pass, and neither is a feature that needs a manual step or silently loses
data after a deploy.

In a harness with subagents, the main session does not review itself: it spawns
`oiko-reviewer` with the spec or target and relays the verdict.

Build or run what is judged from a detached worktree of the merged base
(`git -C <repo> worktree add --detach <repo>/.worktrees/<slug>-review <base>`);
remove it and stop your processes when done. Evidence goes under
`~/Development/agent-state/<slug>/review/`. Load a repo's dev skill only for the
repos the change touched.

## Acceptance against a spec

1. **Each acceptance criterion:** exercise it the way a user or API consumer
   would and record the evidence it names. Verdict: pass, fail or partial, with
   a reason.
2. **Owner decisions** in the spec were honoured, not reinterpreted.
3. **Behaviour section in full:** web and desktop, loading, empty, error and
   offline states, restart and redeploy.
4. **Contract, when Copepod's API changed** (see `oiko-contract`):
   - server: route registered, handler tested for auth failure and tenant/app
     isolation, `docs/api-integration-reference.md` updated, OpenAPI surface
     `--check` passes;
   - SDK: method matches the documented path, auth header type and body; a
     wiremock test asserts them, including the error path;
   - Oikonotes: pin points at a pushed SDK rev, calls go through
     `CopepodBridge`, and the change classification (additive / breaking) holds.
5. **Invariants:**
   - no persistent user data through `state.pool` / `legacy_sqlite_pool()`
     (web mode wipes it on restart); every new domain has a store trait with
     both Copepod and local implementations;
   - no silent AI writes: suggestions stay explicit apply/reject;
   - Oikonotes: the spec's Identity answers hold in the built code
     (`docs/product-identity.md` charter check) — privacy flags excluded from
     every AI, recall and publishing path; permanent and index notes stay
     prompt-only; provenance survives apply;
   - platform auth and app-user auth never mix; every query scoped by org, app
     and user;
   - route content is never an `#[island]`; pages load data with
     `Resource` + `<Suspense>`;
   - no secrets in code, docs, logs or fixtures.
6. **Regressions next door:** run the tests and Playwright specs covering the
   touched surfaces; compare with the base commit before blaming the feature.
   Spend your time on reading and probing, not on repeating the gate: read the
   implementer's gate logs, and re-run the full gate only when they are missing,
   red, or older than a change to code they cover. A clean-tree run of the
   focused tests is cheap and catches flakes.
7. **UI:** screenshots at desktop (1280×800) and mobile (390×844); keyboard
   focus, labels, no horizontal overflow (`docs/mobile-conventions.md`).

Report a verdict table (one row per criterion), then defects ranked by user
impact, each with repo, repro steps, evidence path and the smallest fix, phrased
so it can become a slice. End with: accepted, or not accepted and what remains.

## Spec review (before approval)

Read the draft spec and the code its decisions rest on; build nothing. List
what would be rejected at acceptance: criteria that cannot be checked or name
no evidence; missing loading, empty, error, offline, restart or redeploy
behaviour; conflicts with the invariants above or the Identity answers; data
users already have left unhandled; slices that cannot merge alone or are out of
dependency order; a contract change classified wrongly. Rank by what it would
cost to find later. Do not redesign what the owner decided.

## Branch review (pre-merge)

`oiko-build` runs this on every slice before its full gate, in the slice's own
worktree and without a review worktree. Read `git diff <base>...<branch>`
against its goal and the spec's acceptance criteria for the slice. Look for
correctness bugs, the invariants above, missing tests for changed behaviour,
states the Behaviour section names and the diff does not handle, and
cross-repo fallout (a server shape change without SDK/Oikonotes follow-up).
Probe a doubt with a focused test or a small script against real parsers and
types; run no build, gate or app, and change no file. Rank findings, each with
file:line and the smallest fix; skip style nits the formatter or clippy would
catch. End with: gate it, or fix first.

## UX critique

Capture each screen at both widths and in its empty, loading, populated and
error states. Walk the real workflow and count steps. Judge against existing
Oikonotes pages, `leptos-daisyui` conventions and `docs/mobile-conventions.md`.
Rank by impact (blocking, confusing, inconsistent, cosmetic); each finding shows
the screen, says what is wrong plainly, proposes a concrete change, and marks it
as your expert call or a product question. Offer to turn findings into a spec
with `oiko-plan`.
