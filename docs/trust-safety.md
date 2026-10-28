# Trust and safety (AI3)

Methods live in `src/api/trust.rs` (app: API key and app-user bearer) and
`src/api/trust_admin.rs` (owner: platform bearer, org admin); types in
`src/models/trust.rs` and `trust_review.rs`; `client.app(org, app).trust()` wraps
all of them. Server contract: copepod `docs/api-integration-reference.md`, Trust
and safety.

## Auth

| Method | Auth |
|---|---|
| `moderate`, `record_moderation_outcome` | `X-API-Key`, scope `ai:moderate` |
| `submit_content_report`, `record_abuse_signal` | `X-API-Key`, scope `trust:write` |
| `list_trust_actions` | `X-API-Key`, scope `trust:read` |
| `moderate_as_user`, `my_restrictions` | app-user bearer |
| `list_review_queue`, `resolve_review_item`, `list_trust_decisions`, `mark_decision_wrong`, `list_content_reports`, `get_app_user_trust`, `list_flagged_app_users`, `lift_app_user_restriction`, `suspend_app_user`, `unsuspend_app_user` | platform bearer, org admin |

API-key methods send `X-API-Key` and no `Authorization`; they return `Auth` when
the client has no key.

## Purposes and failing open

`ModerationPurpose` picks the server policy (`comment`, `public_suggestion`,
`publish`, `outbound_email`, `profile`, `ingested_content`, `chat_input`,
`report`, `signup`). A provider outage never fails `moderate`: the answer has
`degraded: true` and the purpose's fail mode. A transport error, 429, 503
(rolling upgrade) or other `Err` is different: the caller decides. The
recommended pattern is the same as the server's, so user-visible content stays
**pending** (`Review`) rather than published, and low-risk input such as a chat
turn may proceed. Do not treat `Err` as `Allow` for anything shown to others.

`subject_ref` is `kind:id` (kind `a-z_` up to 40, id `A-Za-z0-9._:-` up to 160),
for example `blog_comment:<id>`. `text` is at most 20,000 characters (a purpose
may allow less; the server answers 400 `invalid_moderation_request`).
The app-user route ignores `actor` and `reviewer`: the actor is the token's user
and the reviewer is `app`. Only the API-key route may send `reviewer: Operator`,
which puts a `review` decision in the Copepod operator queue.

Edits to a record covered by a `moderate` lifecycle hook are re-moderated by the
server: a blocked edit answers `422 content_rejected` and stores nothing
(`CopepodError::is_content_rejected`).

## Outcomes and wrong decisions

After the app's owner approves or removes a held item, call
`record_moderation_outcome(decision_id, ...)`. It is idempotent; the app's last
write wins, but an operator's resolution is never overwritten. `wrong_decision:
true` labels the original decision as wrong (training data). The note is checked
(500 characters) but not stored. `mark_decision_wrong` is the owner-side
equivalent for an already decided item: for content purposes the answer carries
the corrected `outcome` and the `action` (`restore`/`takedown`) emitted into the
actions feed; a decision still waiting in the queue answers 409
`decision_in_review` (resolve it with `resolve_review_item`).

## Actions feed

`list_trust_actions(after, limit)` returns entries with `seq > after`, oldest
first, at most `limit` (1..=100). **Store `next_after` and pass it as `after` on
the next poll**; with nothing new it echoes your `after`, so polling is safe to
repeat. Apply `takedown`/`restore` by `subject_ref` and `suspend`/`unsuspend`/
`restriction`/`restriction_lifted` by `user_id`. Entries are kept 30 days, so poll
at least daily; a gap longer than that loses actions. Unknown kinds decode to
`TrustActionKind::Unknown`: skip them but still advance the cursor.

## Reports

`submit_content_report` files a report as the app's server: the reporter is
identified only by `reporter_ip_hash` (64 hex characters, a hash of the address,
never the address). The server rate limits per reporter and per subject: 429
`report_rate_limited`. `duplicate: true` means the same reporter reported the
same subject within a day (no new report). `urgent` priority is set by the
server from the category and triage.

## Abuse signals, restrictions, suspension

`record_abuse_signal` reports what only the app can see: `chat_jailbreak`,
`chat_harm`, `rate_limit_hammering`, `invite_cap`. `moderation_block` and
`report_upheld` are recorded by Copepod and answer 400. `evidence_ref` is an id
(at most 200 characters of letters, digits and `_ . : / -`), never content. The
receipt's `restriction` is set when the signal tipped the score over the
threshold. Weights, decay and the threshold are server-side.

A restricted user's hosted AI answers 403 `ai_paused_for_review`; read the end
of the pause with `CopepodError::ai_paused_for_review_details()` (`scope`,
`until`). A suspended user's calls answer 403 `account_suspended`
(`is_account_suspended`) and `my_restrictions` still works to tell them why.
`suspend_app_user` answers 409 `already_suspended`; `unsuspend_app_user` 409
`not_suspended`; `lift_app_user_restriction` (scopes `hosted_ai`,
`public_posting`, `invites` only) 404 `restriction_not_found`.

## Errors added

`ACCOUNT_SUSPENDED_CODE`, `REPORT_RATE_LIMITED_CODE`,
`MODERATION_RATE_LIMITED_CODE`, `CONTENT_REJECTED_CODE`, `ALREADY_SUSPENDED_CODE`,
`NOT_SUSPENDED_CODE`, `RESTRICTION_NOT_FOUND_CODE`, each with an `is_*` helper
on `CopepodError`. They arrive as `CopepodError::Api`; `ai_paused_for_review`
arrives as `ApiWithDetails` (its `scope` and `until` sit at the top level).

## Enums

Every trust enum is `#[non_exhaustive]` with an `Unknown` variant that absorbs a
value from a newer server. `Unknown` is never sent: request methods reject it with
`InvalidArgument` before the network.
