use copepod_sdk::{CopepodClient, CopepodError, CreditPackPurchaseRequest};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ME: &str = "/api/platform/orgs/org/apps/app/auth/users/me/credits";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn purchase_json(status: &str) -> serde_json::Value {
    json!({"purchase_id": "p1", "pack_id": "credits_300", "credits": 300, "status": status,
           "checkout_url": "https://pay.example/c", "created": "2026-10-17T10:00:00Z",
           "paid_at": null})
}

fn req() -> CreditPackPurchaseRequest {
    CreditPackPurchaseRequest {
        redirect_url: " https://app.example/back ".into(),
    }
}

#[tokio::test]
async fn list_packs_decodes_catalog() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/packs")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"packs": [
            {"id": "credits_300", "credits": 300, "price_minor": 500, "currency": "GBP"},
            {"id": "credits_650", "credits": 650, "price_minor": 1000, "currency": "GBP"}]})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server)
        .list_packs("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(c.packs.len(), 2);
    assert_eq!((c.packs[1].credits, c.packs[1].price_minor), (650, 1000));
    let r = &server.received_requests().await.unwrap()[0];
    assert!(r.headers.get("x-api-key").is_none());
}

#[tokio::test]
async fn purchase_sends_key_trimmed_body_and_decodes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/packs/credits_300/purchase")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "k-1"))
        .and(body_json(
            json!({"redirect_url": "https://app.example/back"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(purchase_json("pending")))
        .expect(1)
        .mount(&server)
        .await;
    let p = client(&server)
        .purchase_credit_pack("org", "app", "users", "credits_300", "k-1", &req())
        .await
        .unwrap();
    assert_eq!(p.purchase_id, "p1");
    assert_eq!(p.status, "pending");
    assert_eq!(p.checkout_url.as_deref(), Some("https://pay.example/c"));
    assert_eq!(p.paid_at, None);
}

#[tokio::test]
async fn purchase_error_codes_map() {
    for (status, code) in [
        (422, "redirect_url_not_allowed"),
        (404, "pack_unavailable"),
        (409, "idempotency_key_reused"),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "no"})),
            )
            .mount(&server)
            .await;
        let err = client(&server)
            .purchase_credit_pack("org", "app", "users", "credits_300", "k-1", &req())
            .await
            .unwrap_err();
        match &err {
            CopepodError::Api { status: s, .. } => assert_eq!(*s, status),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(err.api_code(), Some(code));
    }
}

#[tokio::test]
async fn purchase_validates_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    let blank = CreditPackPurchaseRequest {
        redirect_url: "  ".into(),
    };
    for r in [
        c.purchase_credit_pack("org", "app", "users", "credits_300", "", &req())
            .await,
        c.purchase_credit_pack("org", "app", "users", "credits_300", "bad key", &req())
            .await,
        c.purchase_credit_pack("org", "app", "users", "a/b", "k", &req())
            .await,
        c.purchase_credit_pack("org", "app", "users", "credits_300", "k", &blank)
            .await,
    ] {
        assert!(matches!(r, Err(CopepodError::InvalidArgument(_))));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn list_purchases_sends_paging_and_decodes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/purchases")))
        .and(query_param("limit", "10"))
        .and(query_param("offset", "20"))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [purchase_json("paid")], "total": 21, "limit": 10, "offset": 20})))
        .expect(1)
        .mount(&server)
        .await;
    let l = client(&server)
        .list_credit_purchases("org", "app", "users", Some(10), Some(20))
        .await
        .unwrap();
    assert_eq!((l.total, l.limit, l.offset, l.items.len()), (21, 10, 20, 1));
    assert_eq!(l.items[0].status, "paid");
}

#[tokio::test]
async fn list_purchases_without_paging_sends_no_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/purchases")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"items": [], "total": 0, "limit": 50, "offset": 0})),
        )
        .mount(&server)
        .await;
    client(&server)
        .list_credit_purchases("org", "app", "users", None, None)
        .await
        .unwrap();
    let r = &server.received_requests().await.unwrap()[0];
    assert_eq!(r.url.query(), None);
}

#[tokio::test]
async fn get_purchase_decodes_and_404_maps() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/purchases/p1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "purchase_id": "p1", "pack_id": "credits_300", "credits": 300, "status": "paid",
            "created": "2026-10-17T10:00:00Z", "paid_at": "2026-10-17T10:05:00Z"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/purchases/nope")))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"code": "not_found", "message": "purchase not found"})),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let p = c
        .get_credit_purchase("org", "app", "users", "p1")
        .await
        .unwrap();
    assert_eq!(p.checkout_url, None);
    assert_eq!(p.paid_at.as_deref(), Some("2026-10-17T10:05:00Z"));
    let err = c
        .get_credit_purchase("org", "app", "users", "nope")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Api { status: 404, .. }));
    assert!(matches!(
        c.get_credit_purchase("org", "app", "users", "").await,
        Err(CopepodError::InvalidArgument(_))
    ));
}
