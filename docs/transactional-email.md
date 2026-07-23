# Transactional email

Transactional email delivery is app-scoped and requires an application API key
with the `email:send` scope. Configure it separately from platform or app-user
Bearer tokens:

```rust,no_run
use copepod_sdk::{CopepodClient, TransactionalEmailRequest};

# async fn example() -> Result<(), copepod_sdk::CopepodError> {
let client = CopepodClient::builder()
    .base_url("https://copepod.example")
    .api_key("cpd_...")
    .build()?;

let email = client.app("org_123", "app_456").email();
let queued = email
    .enqueue(
        "trip-invite:participant_789:v1",
        &TransactionalEmailRequest {
            to: "traveler@example.com".into(),
            subject: "You are invited".into(),
            body_html: "<p>Join the trip.</p>".into(),
        },
    )
    .await?;

let delivery = email.get(&queued.delivery_id).await?;
println!("{:?}", delivery.status);
# Ok(())
# }
```

The enqueue call returns after Copepod durably records the delivery; it does not
wait for the provider. Idempotency keys must contain 1–128 visible ASCII
characters. Retry the same request with the same key after an ambiguous network
failure. Copepod returns the original delivery for identical requests and a
conflict if the key is reused with different content.

The SDK validates the recipient, subject, HTML size, idempotency key, and
resource identifiers before issuing a request. API keys are rejected if they
cannot be represented safely as an HTTP header, are marked sensitive in the
request, and are redacted from the client's `Debug` output.

Use `statuses` to fetch between 1 and 100 deliveries in one request. A terminally
`failed` delivery can be retried. A `pending` or `failed` delivery can be
cancelled; attempting to cancel a `sending` or `sent` delivery returns a
conflict. Delivery responses contain state and sanitized errors only; recipient
and message content are never returned.

Transactional email methods always use `X-API-Key`. They do not send an
`Authorization: Bearer` header, even when the same client also has a Bearer
token configured for other Copepod operations.
