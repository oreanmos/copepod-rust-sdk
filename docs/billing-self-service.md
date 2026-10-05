# Billing self-service (B1)

Signed-in app users use the app-user bearer token (never the API key).

- `me_start_trial(org, app, collection, &MeTrialStartRequest)` posts
  `billing_country` and `evidence.ip_country` to `.../me/subscription/trial`.
  Retry is safe: a second call returns 409 `trial_already_used` or
  `subscription_exists`. Other codes: 422 `billing_country_not_supported`,
  `trial_unavailable` (also returned in card-trial mode, where no no-card trial
  exists).
- `me_start_checkout(org, app, collection, key, &MeCheckoutRequest)` posts to
  `.../me/subscription/checkout` with an `Idempotency-Key`. Persist the key
  before the call; reuse it with the unchanged body after a lost response to get
  the same `payment_id`/`checkout_url`. Send the user to `checkout_url`; the
  subscription becomes active when the payment webhook is processed. Codes:
  422 `billing_interval_unavailable`, `billing_country_not_supported`,
  `discount_code_invalid`, `discount_code_exhausted`; 409 `subscription_active`
  (paid subscription already active or in grace, or a live Mollie subscription
  including a card trial); 409 when the key was already used with a different
  body; 409 `checkout_pending` while an earlier checkout is open and its Mollie
  payment has not expired (a replay with the same key and body still returns the
  original response). The `checkout_pending` body has `checkout_url` (nullable)
  and `expires_at` (RFC 3339: the payment's Mollie `expiresAt`, or the session's
  created time plus 6 h) at the top level; the error is
  `CopepodError::ApiWithDetails` and `err.checkout_pending_details()` returns
  `CheckoutPending { checkout_url, expires_at }` so the app can send the user
  back to the open payment. `api_code()` and `api_status()` work as before; 400 when `redirect_url` is not an absolute HTTPS URL
  (`http` is accepted for localhost only). Current servers require
  `redirect_url` (missing or empty is a 400 with no error code); the SDK keeps it `Option<String>`
  for older servers, so always send it. `promo_code` and unset optional fields are omitted
  from the body when `None`. A bad key or blank `redirect_url` fails offline
  with `InvalidArgument`.
- Pre-registration intents gain `billing_interval`, `billing_country`, `evidence`
  (`BillingIntentCreate` now implements `Default`); `AppBillingCatalog.signup`
  carries the public signup policy.
- `app_register` and `scoped.auth().register` return `AppRegisterResponse`
  (`token`, `refresh_token`, `record`, optional `trial`), matching the server;
  pass an `AppRegisterRequest` as the body.
- Older servers return 404 for both `me/subscription` routes and omit the new
  fields; the SDK does not fall back.

## Billing page, receipts and payment recovery

All four calls use the app-user bearer token (never the API key) and live under
`api/platform/orgs/{org}/apps/{app}/auth/{collection}/me/billing`. Older servers
return 404 for all of them; the SDK does not fall back. A token from another app
is a 401.

### Billing page

`me_billing_summary` returns `MeBillingSummary`: `status` (`none`, `trialing`,
`active`, `past_due`, `cancelled`, `expired`) plus plan, period, cancel-at-period-end,
pending plan change and, when past due, `past_due` (amount, grace end,
`recovery_checkout_open`). `next_charge` is null unless a Mollie subscription
exists. `MePaymentMethod.method` is Mollie's value or `None`.

### Receipts

`me_list_receipts(limit, offset)` returns `MeReceiptList`, newest first. `limit`
must be 1-100 and `offset` 0 or more; the SDK rejects others offline with
`InvalidArgument` (the server answers 400), and sends a parameter only when `Some`.
`me_get_receipt_html(receipt_id)` returns the standalone HTML page as a `String`;
an unknown or foreign receipt is 404 `receipt_not_found`.

### Payment recovery

`me_start_payment_recovery(key, body)` sends an `Idempotency-Key` (same rules as
checkout) and `redirect_url` (non-empty), and returns the checkout for the
outstanding amount (`payment_id`, `checkout_url`, `amount_cents`, `currency`,
`grace_ends_at`). Errors: 409 `not_past_due`, `recovery_unavailable`,
`recovery_processing` (poll `me_billing_summary` until active), 422
`redirect_url_not_allowed`. While past due, `me_start_checkout` answers 409
`subscription_past_due`; use recovery instead.
