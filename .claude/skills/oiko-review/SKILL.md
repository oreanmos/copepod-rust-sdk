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
`~/.cache/oiko-agents/<slug>/review/`. Load a repo's dev skill only for the
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
   - platform auth and app-user auth never mix; every query scoped by org, app
     and user;
   - route content is never an `#[island]`; pages load data with
     `Resource` + `<Suspense>`;
   - no secrets in code, docs, logs or fixtures.
6. **Regressions next door:** run the tests and Playwright specs covering the
   touched surfaces; compare with the base commit before blaming the feature.
7. **UI:** screenshots at desktop (1280×800) and mobile (390×844); keyboard
   focus, labels, no horizontal overflow (`docs/mobile-conventions.md`).

Report a verdict table (one row per criterion), then defects ranked by user
impact, each with repo, repro steps, evidence path and the smallest fix, phrased
so it can become a slice. End with: accepted, or not accepted and what remains.

## Branch review (pre-merge)

Read `git diff <base>...<branch>` against its goal. Look for correctness bugs,
the invariants above, missing tests for changed behaviour, and cross-repo
fallout (a server shape change without SDK/Oikonotes follow-up). Rank findings;
skip style nits the formatter or clippy would catch.

## UX critique

Capture each screen at both widths and in its empty, loading, populated and
error states. Walk the real workflow and count steps. Judge against existing
Oikonotes pages, `leptos-daisyui` conventions and `docs/mobile-conventions.md`.
Rank by impact (blocking, confusing, inconsistent, cosmetic); each finding shows
the screen, says what is wrong plainly, proposes a concrete change, and marks it
as your expert call or a product question. Offer to turn findings into a spec
with `oiko-plan`.
