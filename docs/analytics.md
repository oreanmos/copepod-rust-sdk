# Analytics: `track_events` and registration attribution

`track_events` posts a batch of app events with the app's API key
(`X-API-Key`, scope `analytics:write`); no Bearer token is sent. Call it from
the app's **server** only: the key is a secret, and the visitor's IP and user
agent are read there.

```rust
use copepod_sdk::{event_names, AnalyticsEvent, CopepodClient};

let client = CopepodClient::builder()
    .base_url("https://admin.copepod.app")
    .api_key(key)                 // scope analytics:write
    .build()?;

let events = [AnalyticsEvent {
    name: event_names::PAGE_VIEW.to_string(),
    occurred_at: chrono::Utc::now(),
    user_id: None,
    path: Some("/pricing".into()),
    referrer: None,
    ip: Some(client_ip),          // as the SSR layer sees it, behind Cloudflare
    user_agent: Some(user_agent),
    acquisition: None,
    props: None,
}];
let response = client.track_events(org_id, app_id, &events).await?;
```

Or `client.app(org_id, app_id).analytics().track(&events)`.

## Who emits which event

Copepod refuses a name it emits itself, so nothing is counted twice.

| Name | Emitted by | When |
|---|---|---|
| `page_view` | the app's server | every page render, with `ip`, `user_agent`, `path`, `referrer` and, on the landing hit, `acquisition` |
| `signup_started` | the app's server | the signup form is first shown in a session |
| `activated` | the app's server | a signed-up user reaches the app's activation moment; `user_id` is required |
| `checkout_started` | the app's server | a checkout is opened, with `user_id` |
| `custom:<name>` | the app's server | anything else the app wants to count; `event_names::custom("owner_check")` validates the name |
| `signup_completed`, `trial_started`, `lead_created`, `lead_confirmed`, `subscription_started`, `subscription_renewed`, `subscription_cancelled`, `subscription_expired`, `payment_failed` | Copepod | registration, leads and the billing watcher; an app that sends one gets `reserved_event` |

`event_names::APP_EVENT_NAMES` and `event_names::COPEPOD_EVENT_NAMES` list both
sets. A custom name is `custom:` plus `^[a-z][a-z0-9_]{0,47}$`.

## Batching

Send **up to 100 events per call** and flush at least **every 10 seconds**;
buffer in the server process and send on whichever comes first. One batch is
one request, and the response reports each event's fate, so a partial failure
never loses the rest. `202` is success even when single events inside the batch
were rejected.

## `ip` and `user_agent`

Send both from the server, with the values the SSR layer actually saw (behind
Cloudflare, the client IP and the `User-Agent` header). Never read them in the
browser and never send them from client-side code: the key would leak, and a
browser-supplied IP is meaningless.

Copepod hashes the pair with today's secret salt into an anonymous daily
visitor id and **stores neither the IP nor the user agent**. The hash cannot
link a visitor across days. Without `ip` (or without a parsable one) no visitor
hash is computed, so the event counts in the totals but not as a visitor.
Events whose user agent matches the bot list, and events from excluded (test or
synthetic) users, are dropped and counted in `filtered`.

## The 48-hour lateness window

`occurred_at` must fall within **[now − 48 hours, now + 5 minutes]**; the
server rejects an event outside it with `occurred_at_out_of_range` (or
`invalid_occurred_at` when it is not RFC 3339). The window is what makes a
retried or buffered batch safe: a flush that failed and is retried an hour
later still lands, and the day it counts in is the event's UTC day, not the
receipt day. It also bounds backfill: a batch older than 48 hours is refused
event by event, so import historical data another way or accept the loss.

Rollups recompute the last three days every 5 minutes, so a late event shows up
in the KPIs within minutes of being accepted.

## Offline validation

`track_events` validates before any request and returns
`CopepodError::InvalidArgument` without touching the network when:

- the batch is empty or holds more than 100 events;
- a `name` is neither one of the four app names nor a valid `custom:<name>`
  (a Copepod-only name is refused with a message naming Copepod as its source);
- a `user_id` is not 1–128 bytes of `[A-Za-z0-9_-]`;
- `props` holds more than 10 keys, or a value that is not a string, a number or
  a bool.

The server applies further per-event rules (`path` starts with `/` and is at
most 512 bytes, prop keys match `^[a-z][a-z0-9_]{0,39}$`, string values at most
200 bytes, the serialised props at most 1024 bytes, `activated` needs a
`user_id`) and rejects the offending event alone with its code.

Without an API key the call fails with `CopepodError::Auth` before any request.

## Response and errors

```json
{"accepted": 97, "filtered": 2, "rejected": [{"index": 4, "code": "invalid_props"}]}
```

`index` is the position in the batch you sent. Match on `code`, not the
message: `reserved_event`, `unknown_event`, `invalid_occurred_at`,
`occurred_at_out_of_range`, `invalid_user_id`, `user_id_required`,
`invalid_path`, `invalid_props`. A whole-batch failure arrives as
`CopepodError::Api { status, code, message }`: 400 `batch_empty` or
`batch_too_large` (the SDK refuses both offline), 401/403 (no key, missing
scope, wrong or inactive app), 404 (app not in the org) and 503 during a
rolling upgrade. Retry a 5xx with the same events: the 48-hour window keeps the
retry valid.

## Registration attribution

`RegisterWithBillingIntentRequest` and `AppRegisterRequest` take
`acquisition: Option<LeadAcquisition>` — first touch, as carried by the site's
links (`utm_*`, `ref`, `referrer_host`, `landing_path`). It is omitted from the
JSON when `None`, and invalid fields are dropped by the server, never an error.
Copepod stores it on the user's growth profile and emits `signup_completed`
itself; the response is unchanged. Both structs derive `Default`, so code that
builds them literally adds `acquisition: None` or `..Default::default()`.
