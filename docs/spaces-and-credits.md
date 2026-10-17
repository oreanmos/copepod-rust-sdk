# Spaces and credits

Contract: copepod `docs/api-integration-reference.md` sections 4.4, 5.4, 5.7,
8.1, 10.9a and 20. Everything here is additive: existing calls are unchanged.
Space management routes (`spaces`, members, invites) need an **app-user bearer
token**; API keys and dashboard tokens get 401 there. The record, file and
realtime calls below also send the bearer token and never `X-API-Key`.

## Spaces

`create_space`, `list_spaces`, `get_space`, `rename_space`, `delete_space`,
`list_space_members`, `set_space_member_role`, `remove_space_member`,
`create_space_invite`, `list_space_invites`, `revoke_space_invite`,
`accept_space_invite` (`src/api/spaces.rs`). Only private Spaces can be created;
`create_space` rejects `Commons` client-side. Invite roles are `owner`,
`editor`, `viewer`. A Space the caller does not belong to is 404
`space_not_found` (same as one that does not exist); a role that cannot do the
operation is 403 `insufficient_role` (`err.is_space_not_found()`,
`err.is_insufficient_role()`; other codes are constants in `error.rs`).

## Records and files in a Space

Space-scoped collections need `X-Space-Id`; without it the server answers 400
`space_required`.

| Call | How |
|---|---|
| list / get | `client.records(org, app, coll).space(id).list()` / `.get_one(rid)` |
| create | `create_record_in_space` |
| update | `update_record_with(.., &RecordUpdateOptions { space_id, if_match })` |
| delete | `delete_record_in_space` |
| files | `upload_file_in_space`, `download_file_in_space`, `delete_file_in_space` |
| realtime | `subscribe_space` (`?space_id=`); `RecordEvent.space_id` is set on Space events |

The plain `subscribe` never receives Space events. `subscribe_space` keeps the
server's error code on failure (404 `space_not_found`); membership is rechecked
every 60 s and a removed member's stream ends within a minute. Record search is
not exposed by the SDK yet.

## If-Match

`RecordUpdateOptions::if_match` sends `If-Match: <rev>`. The collection must
declare `options.revision_field` (else 400). On a match the server applies the
update and bumps the revision itself; on a stale revision it answers 409
`revision_conflict` and writes nothing: `err.is_revision_conflict()` and
`err.revision_conflict_details()` (`current_rev`, a JSON number). The error is
`CopepodError::ApiWithDetails` (the body is the details); `api_status()` and
`api_code()` work as for every other error. Roll-out: send `If-Match` only
after every node supports it, an older leader ignores the header and the update
would be unconditional.

## Credits

`get_current_credits`, `check_current_credits`, `report_current_credits`
(`me/credits*`). `check` fails with 402 `credits_exhausted`, `report` with 409
`credits_exhausted`; both carry `remaining_total` and `resets_at`:
`err.credits_exhausted_details()`. `report` is idempotent per `request_id`
(an identical retry returns the first answer; `remaining_total` is the balance
now). `kind` is `ai`, `ai:<name>` or `connector:<name>`, checked client-side.
During a rolling upgrade report routes answer 503 `credits_upgrading`. On
servers without the ledger these routes are 404.

`AiUsageStatus` gained `credit_remaining_total` and `credit_topups` (absent on
older servers, `None`); `AiUsageReportRequest` gained `kind` (omitted when
`None`). Both are new struct fields: code that builds `AiUsageReportRequest` or
`RecordEvent` with a struct literal must add them (Oikonotes has no such site).
