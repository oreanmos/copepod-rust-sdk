use copepod_sdk::{
    CopepodClient, CopepodError, CURRENCY_MISMATCH_CODE, MANDATE_UNAVAILABLE_CODE,
    PACKS_UPGRADING_CODE, PAYMENT_PROVIDER_REJECTED_CODE, REQUIRES_PAID_PLAN_CODE,
    STORAGE_CAPACITY_UNAVAILABLE_CODE, STORAGE_UNLIMITED_CODE,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ME: &str = "/api/platform/orgs/org/apps/app/auth/users/me/storage-packs";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

#[tokio::test]
async fn list_decodes_catalog_owned_and_capacity() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ME))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "storage_packs": [{"id": "storage_1gb", "gb": 1, "price_minor": 100,
                               "currency": "GBP", "interval": "month"}],
            "owned_storage": [
                {"subscription_id": "s1", "pack_id": "storage_1gb", "quantity": 5,
                 "status": "active", "renews_at": "2026-11-25T00:00:00Z"},
                {"subscription_id": "s2", "pack_id": "storage_1gb", "quantity": 1,
                 "status": "pending", "renews_at": null}],
            "capacity_open": false})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server)
        .list_storage_packs("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(c.storage_packs[0].gb, 1);
    assert_eq!(c.storage_packs[0].interval, "month");
    assert_eq!(c.owned_storage[0].quantity, 5);
    assert_eq!(c.owned_storage[1].renews_at, None);
    assert!(!c.capacity_open);
    let r = &server.received_requests().await.unwrap()[0];
    assert!(r.headers.get("x-api-key").is_none());
}

#[tokio::test]
async fn subscribe_sends_key_and_quantity_and_decodes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/storage_1gb/subscribe")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "k-1"))
        .and(body_json(json!({"quantity": 3})))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"pack_subscription_id": "s1", "status": "pending", "ends_at": null}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let s = client(&server)
        .subscribe_storage_pack("org", "app", "users", "storage_1gb", 3, "k-1")
        .await
        .unwrap();
    assert_eq!(
        (s.pack_subscription_id.as_str(), s.status.as_str()),
        ("s1", "pending")
    );
    assert_eq!(s.ends_at, None);
}

#[tokio::test]
async fn subscribe_validates_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    for r in [
        c.subscribe_storage_pack("org", "app", "users", "storage_1gb", 0, "k")
            .await,
        c.subscribe_storage_pack("org", "app", "users", "storage_1gb", 101, "k")
            .await,
        c.subscribe_storage_pack("org", "app", "users", "storage_1gb", 1, "")
            .await,
        c.subscribe_storage_pack("org", "app", "users", "storage_1gb", 1, "bad key")
            .await,
        c.subscribe_storage_pack("org", "app", "users", "a/b", 1, "k")
            .await,
    ] {
        assert!(matches!(r, Err(CopepodError::InvalidArgument(_))));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn subscribe_error_codes_map() {
    for (status, code, constant) in [
        (402, "requires_paid_plan", Some(REQUIRES_PAID_PLAN_CODE)),
        (409, "currency_mismatch", Some(CURRENCY_MISMATCH_CODE)),
        (409, "storage_unlimited", Some(STORAGE_UNLIMITED_CODE)),
        (
            409,
            "capacity_unavailable",
            Some(STORAGE_CAPACITY_UNAVAILABLE_CODE),
        ),
        (409, "mandate_unavailable", Some(MANDATE_UNAVAILABLE_CODE)),
        (
            502,
            "payment_provider_rejected",
            Some(PAYMENT_PROVIDER_REJECTED_CODE),
        ),
        (503, "packs_upgrading", Some(PACKS_UPGRADING_CODE)),
        (404, "pack_unavailable", None),
        (409, "idempotency_key_reused", None),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "no"})),
            )
            .mount(&server)
            .await;
        let err = client(&server)
            .subscribe_storage_pack("org", "app", "users", "storage_1gb", 1, "k-1")
            .await
            .unwrap_err();
        match &err {
            CopepodError::Api { status: s, .. } => assert_eq!(*s, status),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(err.api_code(), Some(code));
        if let Some(constant) = constant {
            assert_eq!(constant, code);
        }
    }
}

#[tokio::test]
async fn cancel_deletes_by_subscription_id_and_decodes() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("{ME}/s1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "pack_subscription_id": "s1", "status": "cancelling",
            "ends_at": "2026-11-25T00:00:00Z"})))
        .expect(1)
        .mount(&server)
        .await;
    let s = client(&server)
        .cancel_storage_pack("org", "app", "users", "s1")
        .await
        .unwrap();
    assert_eq!(s.status, "cancelling");
    assert_eq!(s.ends_at.as_deref(), Some("2026-11-25T00:00:00Z"));
    let r = &server.received_requests().await.unwrap()[0];
    assert!(r.headers.get("x-api-key").is_none());
}

#[tokio::test]
async fn cancel_maps_404_and_503_and_validates_id() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("{ME}/nope")))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"code": "not_found", "message": "storage pack not found"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{ME}/busy")))
        .respond_with(
            ResponseTemplate::new(503)
                .set_body_json(json!({"code": "packs_upgrading", "message": "later"})),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c
        .cancel_storage_pack("org", "app", "users", "nope")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Api { status: 404, .. }));
    let err = c
        .cancel_storage_pack("org", "app", "users", "busy")
        .await
        .unwrap_err();
    assert_eq!(err.api_code(), Some(PACKS_UPGRADING_CODE));
    assert!(matches!(
        c.cancel_storage_pack("org", "app", "users", "").await,
        Err(CopepodError::InvalidArgument(_))
    ));
}
