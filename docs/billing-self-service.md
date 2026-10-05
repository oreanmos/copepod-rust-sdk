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
  (paid subscription already active or in grace); 409 when the key was already
  used with a different body; 409 when an open checkout exists (see the server
  docs for its code); 400 when `redirect_url` is not an absolute HTTPS URL
  (`http` is accepted for localhost only). Current servers require
  `redirect_url` (missing or empty is 400); the SDK keeps it `Option<String>`
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
