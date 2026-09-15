# Recovering billing intent creation

Use `create_app_billing_intent_idempotent(org_id, app_id, key, &request)` when
creating a pre-registration billing intent. Persist a random operation key before
the first call, and reuse the same key and request after a transport failure or
lost response. The SDK sends the public request without platform or app credentials.

Keys contain 1–160 ASCII letters, digits, `.`, `:`, `_`, or `-`. The server scopes
keys by organization and app. An identical request recovers the existing intent;
changing the request with the same key returns HTTP 409. The SDK propagates that
conflict and never automatically retries or substitutes a new key.

The server must support this contract before a client relies on it. The legacy
`create_app_billing_intent` method remains available and unchanged.

Validation: 86 SDK tests passed, zero failed or ignored; strict all-target Clippy
passed. Server-side durable replay, concurrent creation, and tenant isolation are
validated separately in Copepod. No payment provider call is made by SDK tests.
