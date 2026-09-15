use copepod_sdk::{BillingIntentCreate, CopepodClient, CopepodError};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn body() -> BillingIntentCreate {
    BillingIntentCreate {
        email: "launch@example.invalid".into(),
        plan_slug: "pro".into(),
        addon_keys: vec![],
        promo_code: None,
        success_url: None,
        cancel_url: None,
        collection: None,
    }
}

#[tokio::test]
async fn repeated_calls_send_the_same_public_request_key_without_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/org/apps/app/billing/intents"))
        .and(header("idempotency-key", "stable-operation"))
        .and(body_json(serde_json::to_value(body()).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id":"intent", "org_id":"org", "app_id":"app", "email":"launch@example.invalid",
            "plan_slug":"pro", "addon_keys":[], "amount_cents":1000, "currency":"EUR",
            "status":"draft", "created":"2026-09-15", "updated":"2026-09-15"
        })))
        .expect(2)
        .mount(&server)
        .await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("must-not-send")
        .api_key("also-must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap();
    for _ in 0..2 {
        assert_eq!(
            client
                .create_app_billing_intent_idempotent("org", "app", "stable-operation", &body())
                .await
                .unwrap()
                .id,
            "intent"
        );
    }
    for request in server.received_requests().await.unwrap() {
        assert!(!request.headers.contains_key("authorization"));
        assert!(!request.headers.contains_key("x-api-key"));
    }
}

#[tokio::test]
async fn conflict_is_returned_without_retrying_or_dropping_the_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("idempotency-key", "same-key"))
        .respond_with(
            ResponseTemplate::new(409).set_body_json(json!({"message":"different request"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .build()
        .unwrap();
    assert!(matches!(
        client
            .create_app_billing_intent_idempotent("org", "app", "same-key", &body())
            .await,
        Err(CopepodError::Api { status: 409, .. })
    ));
}

#[tokio::test]
async fn invalid_keys_fail_before_sending() {
    let server = MockServer::start().await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .build()
        .unwrap();
    for key in ["", "bad key", &"a".repeat(161)] {
        assert!(matches!(
            client
                .create_app_billing_intent_idempotent("org", "app", key, &body())
                .await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
