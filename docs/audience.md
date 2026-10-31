# Audience: leads and announcements

Both calls use the app's API key (`X-API-Key`) and never send a Bearer token.
Call them from server code only: the key is a secret.

## Leads: `create_lead`

`CopepodClient::create_lead(org_id, app_id, &CreateLeadRequest)` or
`client.app(org, app).audience().create_lead(&lead)`. The key needs scope
`leads:write`.

- **202 and no enumeration.** Every well-formed request answers
  `{"accepted": true}` whether the address is new, pending, confirmed or
  previously unsubscribed. Always show the same "check your inbox" message.
  Double opt-in: the server mails a confirmation link (at most 5 per lead, 10
  minutes apart).
- **Consent.** Pass the exact sentence the visitor saw as `consent_text`
  (1-500 characters, no NUL). `list_key` matches `^[a-z][a-z0-9_]{0,39}$`;
  `country` is trimmed and upper-cased to an ISO 3166-1 alpha-2 code (`gb` is
  sent as `GB`). The SDK mirrors the server's `invalid_lead` rules first,
  trimming as it does: email shape, `client_ip` parses as one IP address (an
  `X-Forwarded-For` list is rejected: pass its first hop), acquisition fields
  at most 200 bytes with no control characters, `landing_path` starts with `/`.
  A violation returns `CopepodError::InvalidArgument` without a request.
- **`client_ip`.** Pass the visitor's IP. The server rate-limits 10 sign-ups an
  hour per `(app, client_ip)`; without it, per `(app, connection peer IP)`
  (120 an hour), which for an SSR server is one bucket for all visitors. It is
  used for rate limiting and Turnstile only and never stored.
- **Turnstile.** When the app enables it, send the widget token as
  `turnstile_token`. A missing or rejected token is 422 `turnstile_failed`
  (ask the visitor to retry the challenge); 503 `turnstile_unavailable` means
  verification could not run.
- **Errors.** All arrive as `CopepodError::Api { status, code, message }`.
  Match on `code`, not the message: 400 `invalid_lead`, 422 `turnstile_failed`,
  429 `rate_limited` ("try later"), 503 `turnstile_unavailable` and 503
  `audience_upgrading` during a rolling upgrade. These carry no `code`: 401, 403
  (missing scope, wrong or inactive app), 404 (app not in the org) and 503 when
  the server's `COPEPOD_EXTERNAL_URL` is unset.

## Announcements: `list_active_announcements`

`list_active_announcements(org_id, app_id)` or `audience().active_announcements()`
returns the announcements whose window contains now (scope
`announcements:read`): at most 20, `critical` first, then `warning`, `info`,
newest first. The response is `Cache-Control: private, max-age=60`; call at
most once a minute per process and cache the vector. `message` is plain text:
render it as text, never as markup. A severity the SDK does not know
(added by a newer server) parses as `AnnouncementSeverity::Unknown` so the rest of
the list still loads; style it like `info`.

## Marketing consent at registration

`RegisterWithBillingIntentRequest` and `AppRegisterRequest` take
`marketing_consent: Option<MarketingConsent>`; it is omitted from the JSON when
`None`. Both structs gained the field and derive `Default`, so code that builds
them literally must add `marketing_consent: None` or `..Default::default()`.
An invalid consent is 400 `invalid_marketing_consent` before the account exists.
`MarketingConsent::validate()` mirrors that rule (trimmed, 1-500 characters, no
NUL); `register_with_billing_intent` calls it before sending, and you should call
it before building an `AppRegisterRequest` body, since `app_register` takes any
serialisable body. See also `billing-self-service.md`.
