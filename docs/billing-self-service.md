# Billing self-service (B1)

Signed-in app users use the app-user bearer token (never the API key).

- `me_start_trial(org, app, collection, &MeTrialStartRequest)` posts
  `billing_country` and `evidence.ip_country` to `.../me/subscription/trial`.
  Retry is safe: a second call returns 409 `trial_already_used` or
  `subscription_exists`. Other codes: 422 `billing_country_not_supported`,
  `trial_unavailable`.
- `me_start_checkout(org, app, collection, key, &MeCheckoutRequest)` posts to
  `.../me/subscription/checkout` with an `Idempotency-Key`. Persist the key
  before the call; reuse it with the unchanged body after a lost response to get
  the same `payment_id`/`checkout_url`. Send the user to `checkout_url`; the
  subscription becomes active when the payment webhook is processed. Codes (422):
  `billing_interval_unavailable`, `billing_country_not_supported`,
  `discount_code_invalid`. `promo_code` and unset optional fields are omitted
  from the body when `None`. A bad key or blank `redirect_url` fails offline
  with `InvalidArgument`.
- Pre-registration intents gain `billing_interval`, `billing_country`, `evidence`
  (`BillingIntentCreate` now implements `Default`); `AppBillingCatalog.signup`
  carries the public signup policy.
- Older servers return 404 for both `me/subscription` routes and omit the new
  fields; the SDK does not fall back.
