# Owner audiences (publishing P8)

A newsletter list per app user (the owner, a blogger in Oikonotes), kept apart from G1's app-wide
leads (`docs/audience.md`). Server contract: copepod `docs/api-integration-reference.md` section 27.
Both clients stay available: `app.audience()` (G1 leads, announcements) and
`app.owner_audience()` (this page). Calling code that wants the raw methods uses the
`put_owner_audience` ... `me_audience_issues` methods on `CopepodClient`.

## Auth: API key or bearer, never both

| Methods | Credential | Scope |
|---|---|---|
| `put_owner_audience` | API key (`X-API-Key`) | `owner_audience:manage` |
| `create_owner_subscriber` | API key | `owner_audience:write` |
| `create_owner_issue`, `test_owner_issue`, `send_owner_issue` | API key | `owner_audience:send` |
| `me_audience`, `me_audience_subscribers`, `me_audience_subscribers_csv`, `me_delete_audience_subscriber`, `me_audience_issues` | the owner's app-user bearer | none (the caller is the owner) |

`owner_audience:*` and `*` cover all three scopes. API-key methods send no `Authorization`
header; `me_*` methods send no `X-API-Key`. The SDK does not model API-key scope constants.

## Semantics

- **202 sign-up.** `create_owner_subscriber` returns `OwnerAccepted { accepted: true }` for every
  well-formed request, whether the address is new, pending, confirmed, unsubscribed or suppressed.
  Show the reader one message in every case. Pass the reader's own IP as `client_ip` (first hop of
  `X-Forwarded-For`): it is the rate-limit key (10 per hour per IP, 200 per hour per list, 429
  `rate_limited`).
- **Test then send.** `test_owner_issue` mails the owner's verified address only (422
  `test_recipient_not_owner`, 429 `test_rate_limited`) and marks the issue tested against its
  content hash. `send_owner_issue` returns 202 and the server's dispatcher does the rest.
- **CSV.** `me_audience_subscribers_csv` returns the filtered rows as text (at most 50,000).
- **Erasure.** `me_delete_audience_subscriber` answers 204 and keeps only a suppression hash.
- **`reply_to_owner`** defaults to `false` (omitted on the wire reads as false). Leave it off until
  the Sweego Reply-To check (server S0) is done.
- **Limits.** `OwnerAudienceLimits` values may be `u64::MAX` for "unlimited"
  (`OwnerAudienceLimits::UNLIMITED`). A plan without the feature reads 0.

## Errors

Coded server errors arrive as `CopepodError::Api { status, code, .. }`; branch on `code`:

| Status | Code |
|---|---|
| 400 | `invalid_owner_audience`, `invalid_subscriber`, `invalid_issue` |
| 404 | `owner_not_found`, `owner_audience_not_found`, `issue_not_found`, `subscriber_not_found` |
| 409 | `subscriber_limit_reached`, `owner_audience_disabled`, `issue_not_tested`, `issue_already_sent`, `issue_limit_reached`, `issue_cancelled` (test or send of a cancelled issue) |
| 422 | `test_recipient_not_owner` |
| 423 | `owner_audience_suspended` |
| 429 | `rate_limited` (`Retry-After: 3600`), `test_rate_limited` |
| 503 | `owner_audience_upgrading` (writes only, until every node runs migration 116; retry later) |

## Client-side validation

Rejected as `CopepodError::InvalidArgument` before any request: path IDs outside
`[A-Za-z0-9_-]{1,128}`; audience display name 1-80 chars, site URL `https://` and at most 200 bytes,
consent text 1-300 chars; subscriber email (at most 254 chars, address-shaped), consent 1-300, client IP
a single IP; issue subject 1-150 chars without line breaks, body 1 byte to 200 KiB without null bytes;
test recipient address-shaped; a subscriber filter with the `Unknown` status. The server sanitises
the issue body, so the SDK does not.

## Forward compatibility

`OwnerAudienceStatus`, `OwnerSubscriberStatus` and `OwnerIssueStatus` are `#[non_exhaustive]` and
decode any status this SDK does not know as `Unknown`; handle it like a neutral, read-only state.
`OwnerIssueStatus::Cancelled` means a suspension or disable cancelled the issue.

## Names

`OwnerAccepted` and `OwnerPage<T>` carry the owner prefix so they do not collide with G1 types or
future generic pages.
