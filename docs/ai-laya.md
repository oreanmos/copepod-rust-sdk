# Laya decide, outcome and donations

Laya is a small private classifier in the Copepod cluster. A **task** (`chat_guard`,
`ugc_screen`, `capture_route`, `ticket_triage`, `error_triage`) is a fixed set of questions about
one kind of text. Server contract: copepod `docs/api-integration-reference.md`, "Laya decide
(AI4 C14)" and "Donations and training data (AI4 C15)". Methods live in `src/api/ai_laya.rs`
(a separate file from `ai.rs` to keep both small), types in `src/models/ai_laya.rs`, scoped
wrappers on `client.app(org, app).ai(collection)`.

## Act only when `confident`

```rust
let answer = client.ai_decide("org", "app", "users",
    &DecideRequest::task("chat_guard", json!({"text": text, "note_title": title}))
        .with_subject_ref("note:01J")).await;
match answer {
    Ok(r) if r.answers.get("risk").is_some_and(|a| a.confident) => { /* act on r.answers["risk"].choice */ }
    _ => { /* use the task's escalation (Tier 2, review or rules) */ }
}
```

- `confident` is true only for a task in mode `active`, on its active checkpoint, with a
  calibration, at or above its threshold. A task in `off` answers 200 with `answers: {}`;
  `shadow` answers but is never `confident`.
- **Any failure is "no answer"**: a 503 `laya_unavailable` (timeout, busy, circuit open), a 404
  from a node older than the routes, a transport error. Apply the task's fail mode.
- `decision_id` is present for every `shadow` call and for sampled `active` calls (about 1 in 20, chosen deterministically from the input: the same input is always or never sampled).
  Keep it to report an outcome. Ad-hoc answers never have one.
- Decide is app overhead: it never charges the user's credits.
- Task `input` fields are cut at the task's `max_chars` by the server, never refused.
  `subject_ref` (the server allows at most 200 characters) is stored only as a hash; the SDK does not check it.

## Two routes, two auth kinds

| Method | Route | Auth |
|---|---|---|
| `ai_decide` | `POST …/auth/{collection}/me/ai/decide` | app-user bearer |
| `ai_decide_with_api_key` | `POST …/apps/{app}/ai/decide` | `X-API-Key` (scope `ai:decide`) only, optional `X-User-Id` |

The API-key methods send no `Authorization` header and work on a client with only an API key.

**Ad-hoc questions are API-key only.** `DecideRequest::adhoc(state, questions)` on the bearer route
is 403 `laya_adhoc_forbidden`; `ai_decide` refuses it with `InvalidArgument` before any request.
An ad-hoc call is one service batch: 1 to 3 `choice` questions with unique ids and a `state` of 1 to
4,000 characters (characters, not bytes). The SDK checks exactly these and nothing more. The answer
has `mode: "adhoc"`, no `decision_id`, `confident: false` and nothing is stored: apply your own
threshold to `confidence`.

## Outcomes

After a cascade (Tier 2 or 3) or the user settles a decision, report the label. Only labels are
sent, never the text; this records agreement for the promotion gate.

- `ai_decide_outcome` (bearer) always records source `user`. The SDK therefore refuses a
  `DecideOutcome` whose source is `tier2` or `tier3` on this route (`DecideOutcome::user(labels)`).
- `ai_decide_outcome_with_api_key` accepts `tier2`, `tier3` and `user`.
- The first label per (decision, source) wins; a repeat succeeds and changes nothing.
- Label keys must be option keys of the decision's task version: 400 `laya_invalid_label`. A
  decision of another app, a missing one or one past 180 days is 404 `laya_decision_unknown`.
  503 `laya_upgrading` means retry later.

## Donations

Off by default; every method uses the app-user bearer.

1. `ai_donations_status` -> `{enabled, count, oldest, policy_version}`.
2. `ai_set_donation_consent(&DonationConsent::grant())` sends policy version
   `DONATION_POLICY_VERSION` (`"2026-10"`). `DonationConsent::revoke()` opts out **and deletes every
   donation** (`deleted` in the result).
3. `ai_donate(&Donation::new(task, text, labels))`: text 1 to 4,000 characters, at least one
   label, optional `language` tag. Only `chat_guard` and `capture_route` take donations. 403
   `laya_donation_consent_required` without consent; 429 `laya_donation_quota` after 2,000
   donations in 30 days (deleting frees quota).
4. `ai_delete_donations` deletes all donations and keeps the consent.

A policy-version bump does not yet ask existing donors to consent again (open owner decision).

## Errors

Codes are constants in `copepod_sdk::error` (`LAYA_*_CODE`, `AI_PAUSED_FOR_REVIEW_CODE`) and arrive
as `CopepodError::Api { status, code, .. }`:

| Status | Code |
|---|---|
| 400 | `laya_invalid_request`, `laya_invalid_label`, `laya_policy_version_unknown`, `laya_task_not_donatable` |
| 403 | `laya_adhoc_forbidden`, `laya_donation_consent_required`, `ai_paused_for_review` (reserved) |
| 404 | `laya_task_unknown`, `laya_decision_unknown` |
| 429 | `laya_donation_quota` |
| 503 | `laya_unavailable`, `laya_upgrading` |

## Older servers

A node older than these routes answers 404 with an empty body. It arrives as
`Api { status: 404, code: None, message: "Unknown error" }`, and `CopepodError::is_route_missing()`
recognises exactly that (a donation UI uses it to hide the toggle). A JSON 404 such as
`laya_task_unknown` or `laya_decision_unknown` is not a missing route.

Unlike `get_ai_models` (AI2), these methods return no separate `Unavailable` value: callers treat
every failure of `decide` as "no answer" (apply the task's fail mode), so nothing more is needed.
