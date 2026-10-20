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
`accept_space_invite` (`src/api/spaces.rs`). `create_space` makes a private
Space or a commons (429 `space_rate_limited` past the daily limits). Invite
roles are checked by the server per Space kind (`contributor` and `moderator`
in a commons). A Space the caller does not belong to is 404
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

## Share links, commons and takedown

Contract: copepod `docs/api-integration-reference.md` (Spaces S2/S3), server
`1dbc341`. All additive. Management calls need the app-user bearer token.

**Visitor shape.** A signed-in non-member of a commons gets `{id, name, kind}`
from `GET spaces/{id}`. `get_space_view` returns `SpaceView::Member` or
`SpaceView::Visitor`; `get_space` keeps returning `SpaceWithRole` and fails
to decode the visitor shape. A private Space stays 404.

**Share links** (`src/api/space_sharing.rs`): `create_share_link`,
`list_share_links`, `revoke_share_link`, `rotate_share_link` (owner or
editor). The token is in the create/rotate answer only (`ShareLinkIssued`);
list never returns it. Scope is `ShareLinkScope::whole_space()` or
`::record(collection, id)`; expiry is RFC 3339 (server default 30 days, at
most 365). Rotation revokes the old link: its token answers 410
`link_revoked`.

`list_shared_records` / `get_shared_record` read through a token and send **no
credentials** (no bearer, no API key): the token in the path is the only
credential. Hidden fields and `created_by` are stripped, `expand` is not
offered (the server refuses it), `per_page` is at most 100. Failed lookups are
throttled per address (429 `link_rate_limited`), so do not probe tokens.

**Suggestions and takedown** (`src/api/space_commons.rs`):
`create_space_suggestion`, `list_space_suggestions` (cursor: pass
`SuggestionPage::next` as `SuggestionListOptions::after`),
`accept_space_suggestion` (returns suggestion and record; 409
`revision_conflict` carries `current_rev`, 409 `suggestion_decided` is final),
`reject_space_suggestion`, `takedown_space_record` (safe to repeat after a
failure). Reason and note are checked client-side (1000 characters, no control
characters), `proposed` must be an object of at most 16 KB.

**Error codes** are constants in `error.rs`: `SPACE_RATE_LIMITED_CODE`,
`SUGGESTION_RATE_LIMITED_CODE`, `SUGGESTION_DECIDED_CODE`,
`SUGGESTION_NOT_FOUND_CODE`, `NOT_COMMONS_CODE`, `COMMONS_UPGRADING_CODE`,
`COMMONS_INCOMPLETE_CODE` (account deletion; repeat it), `SHARE_LINKS_UPGRADING_CODE`,
`LINK_NOT_FOUND_CODE`, `LINK_REVOKED_CODE`, `LINK_EXPIRED_CODE`,
`LINK_RATE_LIMITED_CODE`, `SHARE_LINK_RATE_LIMITED_CODE`, `SHARE_LINK_LIMIT_CODE`,
`INSUFFICIENT_ROLE_CODE`, `REVISION_CONFLICT_CODE`. During a rolling upgrade the
`*_upgrading` 503s are retryable.

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

## Anonymous commons reads and the master switch

Contract: copepod `docs/api-integration-reference.md` 5.7 ("Anonymous commons
reads", "Master switch"), server `d8524de`. All additive.

**Reads** (`src/api/space_anonymous.rs`, no credentials, no bearer and no API
key): `get_public_commons` (`{id, name, kind}` only; the server sends no
description), `list_public_commons_records` (`CommonsRecordsQuery`: `q`,
`filter`, `sort`, `fields`, `page`, `per_page`), `get_public_commons_record`,
`sign_public_commons_file` (`{url, expires}`, a 60 second link, `no-store`).
They answer only while the owner set `anonymous_read` and the app's master
switch is on. A missing, private, non-commons or closed Space (or the switch
off) is the same 404 `space_not_found`; each miss counts against the caller's
address (429 `link_rate_limited`), so do not probe ids. A collection that is
not open to every member, or has no such name, is a plain 404 without a code;
a filter, sort or `fields` naming hidden fields is 400. Answers may be cached
30 seconds, so a just-closed commons can still read for that long.

**Validation before the call** (matches the server): `q` at most 100
characters (it is a literal contains-match; `%`, `_` and `\` match
themselves), a filter with at most 8 comparisons and no `~` / `!~` (quoted
strings are skipped; use `q` to search).

**Owner switch:** `set_space_anonymous_read(space_id, on)` (owner bearer,
`PATCH spaces/{id}`): 400 `not_commons`, 403 `insufficient_role`, 404
`space_not_found`, 409 `anonymous_commons_disabled` (switching on while the
master switch is off, checked after the owner and kind checks), 503
`anonymous_commons_upgrading` during a rolling upgrade.

**Master switch:** `set_anonymous_commons_enabled(org, app, Some(bool) | None)`
patches `settings.anonymous_commons_enabled` (None clears it, default on).
`App` now carries `settings`, and `App::anonymous_commons_enabled()` reads the
switch the way the server does. Off closes every anonymous door and kills file
links already issued (401); turning it on again restores what owners had open.
A node on an older server image ignores the switch until upgraded.
