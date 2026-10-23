use copepod_sdk::{
    BillingIntentCreate, CopepodClient, CopepodError, CAPACITY_REACHED_CODE,
    STORAGE_QUOTA_EXCEEDED_CODE,
};
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn client_for(status: u16, body: serde_json::Value) -> (MockServer, CopepodClient) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*"))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(&server)
        .await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("t")
        .auto_refresh(false)
        .build()
        .unwrap();
    (server, client)
}

fn capacity_body() -> serde_json::Value {
    json!({"status":409,"code":"capacity_reached","error":"capacity_reached",
           "message":"This app is at capacity right now.","limit":"max_active_users"})
}

fn intent() -> BillingIntentCreate {
    BillingIntentCreate {
        email: "a@example.invalid".into(),
        plan_slug: "pro".into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn capacity_reached_maps_to_typed_variant() {
    let (_s, client) = client_for(409, capacity_body()).await;
    let err = client
        .create_app_billing_intent_idempotent("org", "app", "key", &intent())
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::CapacityReached { .. }));
    assert!(err.is_capacity_reached());
    assert_eq!(err.capacity_reached_limit(), Some("max_active_users"));
    assert_eq!(err.api_status(), Some(409));
    assert_eq!(err.api_code(), Some(CAPACITY_REACHED_CODE));
    assert!(err.to_string().contains("at capacity"));

    // The register path decodes through the same mapping.
    let err = client
        .app_register("org", "app", "users", &json!({"email":"a@example.invalid"}))
        .await
        .unwrap_err();
    assert_eq!(err.capacity_reached_limit(), Some("max_active_users"));
}

#[tokio::test]
async fn storage_quota_exceeded_maps_with_numbers() {
    let (_s, client) = client_for(
        422,
        json!({"status":422,"code":"storage_quota_exceeded","error":"storage_quota_exceeded",
               "message":"Storage quota exceeded: 24.9 GB of 25 GB used.",
               "used_bytes":24_900_000_000u64,"limit_bytes":25_000_000_000u64,"file_bytes":200_000_000u64}),
    )
    .await;
    let err = client
        .upload_file("org", "app", "notes", "r1", vec![1], "a.png", "image/png")
        .await
        .unwrap_err();
    assert!(err.is_storage_quota_exceeded());
    assert_eq!(
        err.storage_quota_exceeded_bytes(),
        Some((24_900_000_000, 25_000_000_000, 200_000_000))
    );
    assert_eq!(err.api_code(), Some(STORAGE_QUOTA_EXCEEDED_CODE));
    assert_eq!(err.api_status(), Some(422));
    assert!(err.to_string().contains("24.9 GB of 25 GB"));
}

#[tokio::test]
async fn quota_code_without_numbers_falls_back_to_api() {
    let (_s, client) = client_for(
        422,
        json!({"code":"storage_quota_exceeded","message":"full","used_bytes":1}),
    )
    .await;
    let err = client
        .upload_file("org", "app", "notes", "r1", vec![1], "a.png", "image/png")
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        CopepodError::Api { status: 422, code: Some(ref c), .. } if c == "storage_quota_exceeded"
    ));
    assert!(!err.is_storage_quota_exceeded());

    // capacity_reached without a string `limit` also stays Api.
    let (_s, client) = client_for(409, json!({"code":"capacity_reached","message":"x"})).await;
    let err = client
        .create_app_billing_intent_idempotent("org", "app", "key", &intent())
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Api { status: 409, .. }));
}

#[tokio::test]
async fn other_409_stays_api() {
    let (_s, client) = client_for(
        409,
        json!({"code":"checkout_pending_x","message":"nope","limit":"max_active_users"}),
    )
    .await;
    let err = client
        .create_app_billing_intent_idempotent("org", "app", "key", &intent())
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Api { status: 409, .. }));
    // A capacity code on the wrong status is not typed either.
    let (_s, client) = client_for(422, capacity_body()).await;
    let err = client
        .upload_file("org", "app", "notes", "r1", vec![1], "a.png", "image/png")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Api { status: 422, .. }));
}
