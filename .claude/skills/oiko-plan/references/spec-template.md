# Spec template

Write specs to `docs/plans/YYYY-MM-DD-<slug>.md` in the repo that owns the
user-visible outcome. Keep them decision-complete: someone who was not in the
conversation, on a smaller model, must be able to build every slice from the
spec alone. Plain language wherever the owner reads it. Omit sections that do
not apply.

```markdown
# <Feature name>

Status: draft | approved YYYY-MM-DD
Slug: <slug>
Repos: oikonotes | copepod | copepod-rust-sdk (list each touched)

## Problem and goal
What the owner wants, in their words, and why it matters.

## Baseline
What happens today, with evidence: file:line references, screenshots under
~/.cache/oiko-agents/<slug>/, test output.

## Behaviour
Step by step, what the user sees and does when done: web and desktop, loading,
empty, error and offline states, and what happens after a restart or deploy.

## API contract
Only when Copepod's API changes. Per endpoint: method and path, auth
(platform cookie | app-user bearer | API key + scope), request and response
shapes, error codes, idempotency. Change class: additive | breaking (and the
deprecation plan). SDK method signature. Deploy order.

## Data
Store trait, collection names, record shape, local SQLite migration, and what
happens to data users already have.

## Identity
Oikonotes specs that change what users see or what AI does: answer the seven
charter-check questions in `docs/product-identity.md` (yes/no, one line each).
A "no" needs an owner decision below. Omit for platform-only work.

## Decisions
### Owner
- <question> → <answer>
### Expert
- <decision> → <why, one plain sentence> → <what the user will see>
Rejected alternatives: <one line each>

## Scope
In: …
Out: … (and why)

## Acceptance criteria
Numbered and observable; each says how it is checked:
1. <criterion> — checked by: <test name | Playwright spec + viewport |
   wiremock test | smoke command | screenshot at desktop and mobile width>

## Slices
Ordered by dependency, one repo each, each independently mergeable.
### S1 <name> — <repo>
- Goal:
- Files and symbols:
- Tests, including one that fails before the change:
- Docs to update (API reference, OpenAPI surface, SDK docs/):
- Size-cap notes:
- Acceptance criteria covered: #…

## Risks
Known risks and how the build watches for them. No open questions at approval.
```
