use copepod_sdk::{CopepodClient, CopepodError, MePaymentRecoveryRequest};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

const ME: &str = "/api/platform/orgs/org/apps/app/auth/users/me/billing";

#[tokio::test]
async fn me_billing_summary_uses_bearer_and_parses_past_due() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ME))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "past_due",
            "plan_slug": "pro",
            "plan_name": "Pro",
            "billing_interval": "month",
            "currency": "EUR",
            "trial_ends_at": null,
            "current_period_start": "2026-09-01T00:00:00Z",
            "current_period_end": "2026-10-01T00:00:00Z",
            "cancel_at_period_end": false,
            "access_ends_at": null,
            "next_charge": null,
            "payment_method": {"method": null, "brand": null, "last4": null},
            "pending_plan_change": null,
            "past_due": {
                "amount_cents": 500,
                "currency": "EUR",
                "grace_ends_at": "2026-10-08T00:00:00Z",
                "recovery_checkout_open": true
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let s = client(&server)
        .me_billing_summary("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(s.status, "past_due");
    assert!(s.next_charge.is_none());
    assert!(s.payment_method.unwrap().method.is_none());
    let pd = s.past_due.unwrap();
    assert_eq!(pd.amount_cents, 500);
    assert!(pd.recovery_checkout_open);
}

#[tokio::test]
async fn me_billing_summary_parses_none_state() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ME))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "none", "plan_slug": null, "plan_name": null,
            "billing_interval": null, "currency": null, "trial_ends_at": null,
            "current_period_start": null, "current_period_end": null,
            "cancel_at_period_end": false, "access_ends_at": null,
            "next_charge": null, "payment_method": null,
            "pending_plan_change": null, "past_due": null
        })))
        .mount(&server)
        .await;
    let s = client(&server)
        .me_billing_summary("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(s.status, "none");
    assert!(s.plan_slug.is_none() && s.past_due.is_none());
}

fn receipt_json() -> Value {
    json!({
        "id": "r1", "receipt_number": "OIKO-0001", "paid_at": "2026-09-01T00:00:00Z",
        "kind": "subscription", "description": "Pro monthly", "plan_slug": "pro",
        "billing_interval": "month", "period_start": "2026-09-01T00:00:00Z",
        "period_end": "2026-10-01T00:00:00Z", "amount_cents": 500, "currency": "EUR",
        "refunded_cents": 0, "refunded_at": null, "status": "paid",
        "payment_method": {"method": "creditcard", "brand": "Visa", "last4": "4242"}
    })
}

#[tokio::test]
async fn me_list_receipts_sends_limit_and_offset() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/receipts")))
        .and(wiremock::matchers::query_param("limit", "10"))
        .and(wiremock::matchers::query_param("offset", "20"))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [receipt_json()], "total": 21, "limit": 10, "offset": 20
        })))
        .expect(1)
        .mount(&server)
        .await;
    let list = client(&server)
        .me_list_receipts("org", "app", "users", Some(10), Some(20))
        .await
        .unwrap();
    assert_eq!(list.total, 21);
    assert_eq!(list.items[0].receipt_number, "OIKO-0001");
    assert_eq!(
        list.items[0]
            .payment_method
            .as_ref()
            .unwrap()
            .last4
            .as_deref(),
        Some("4242")
    );
}

#[tokio::test]
async fn me_list_receipts_omits_query_when_none() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/receipts")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [], "total": 0, "limit": 50, "offset": 0
        })))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .me_list_receipts("org", "app", "users", None, None)
        .await
        .unwrap();
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs[0].url.query(), None);
}

#[tokio::test]
async fn me_list_receipts_rejects_limit_over_100_offline() {
    let server = MockServer::start().await;
    for limit in [0, 101, -1] {
        let err = client(&server)
            .me_list_receipts("org", "app", "users", Some(limit), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)));
    }
    let err = client(&server)
        .me_list_receipts("org", "app", "users", None, Some(-1))
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn me_get_receipt_html_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/receipts/r1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html; charset=utf-8")
                .set_body_string("<html><body>Receipt</body></html>"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let html = client(&server)
        .me_get_receipt_html("org", "app", "users", "r1")
        .await
        .unwrap();
    assert_eq!(html, "<html><body>Receipt</body></html>");
    let err = client(&server)
        .me_get_receipt_html("org", "app", "users", "a/b")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
    for bad in ["", "r1?x", "#", ".", "..", "r 1", "r%2F1"] {
        let err = client(&server)
            .me_get_receipt_html("org", "app", "users", bad)
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{bad}");
    }
}

#[tokio::test]
async fn me_get_receipt_html_maps_receipt_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/receipts/nope")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "code": "receipt_not_found", "error": "receipt_not_found",
            "message": "receipt not found"
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .me_get_receipt_html("org", "app", "users", "nope")
        .await
        .unwrap_err();
    assert_eq!(err.api_code(), Some("receipt_not_found"));
    assert!(err.to_string().contains("receipt not found"));
    assert_eq!(err.api_status(), Some(404));
}

fn recovery_body() -> MePaymentRecoveryRequest {
    MePaymentRecoveryRequest {
        redirect_url: "https://oikonotes.com/billing/return".into(),
    }
}

#[tokio::test]
async fn me_start_payment_recovery_sends_key_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/payment-recovery")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "rec-1"))
        .and(body_json(
            json!({"redirect_url": "https://oikonotes.com/billing/return"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "payment_id": "p1",
            "checkout_url": "https://checkout.example.invalid/p1",
            "amount_cents": 500,
            "currency": "EUR",
            "grace_ends_at": "2026-10-08T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .me_start_payment_recovery("org", "app", "users", "rec-1", &recovery_body())
        .await
        .unwrap();
    assert_eq!(r.payment_id, "p1");
    assert_eq!(r.amount_cents, 500);
    assert_eq!(r.grace_ends_at, "2026-10-08T00:00:00Z");
}

#[tokio::test]
async fn me_start_payment_recovery_maps_not_past_due() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/payment-recovery")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "not_past_due", "error": "not_past_due",
            "message": "subscription is not past due"
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .me_start_payment_recovery("org", "app", "users", "rec-1", &recovery_body())
        .await
        .unwrap_err();
    assert_eq!(err.api_code(), Some("not_past_due"));
    assert!(err.to_string().contains("subscription is not past due"));
    assert_eq!(err.api_status(), Some(409));
}

#[tokio::test]
async fn me_start_payment_recovery_rejects_bad_input_offline() {
    let server = MockServer::start().await;
    let c = client(&server);
    let err = c
        .me_start_payment_recovery("org", "app", "users", "bad key", &recovery_body())
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
    let err = c
        .me_start_payment_recovery(
            "org",
            "app",
            "users",
            "rec-1",
            &MePaymentRecoveryRequest {
                redirect_url: " ".into(),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn me_routes_never_send_api_key() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [], "total": 0, "limit": 50, "offset": 0
        })))
        .mount(&server)
        .await;
    let c = client(&server);
    c.me_list_receipts("org", "app", "users", None, None)
        .await
        .unwrap();
    let _ = c.me_get_receipt_html("org", "app", "users", "r1").await;
    let _ = c.me_billing_summary("org", "app", "users").await;
    let _ = c
        .me_start_payment_recovery("org", "app", "users", "k", &recovery_body())
        .await;
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 4);
    for r in reqs {
        assert!(!r.headers.contains_key("x-api-key"));
        assert!(r.headers.contains_key("authorization"));
    }
}

#[tokio::test]
async fn me_start_payment_recovery_maps_recovery_processing_and_redirect_not_allowed() {
    for (status, code, message) in [
        (
            409,
            "recovery_processing",
            "a recovery payment is being processed",
        ),
        (
            422,
            "redirect_url_not_allowed",
            "redirect_url is not allowed",
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(format!("{ME}/payment-recovery")))
            .respond_with(ResponseTemplate::new(status).set_body_json(json!({
                "code": code, "error": code, "message": message
            })))
            .mount(&server)
            .await;
        let err = client(&server)
            .me_start_payment_recovery("org", "app", "users", "rec-1", &recovery_body())
            .await
            .unwrap_err();
        assert_eq!(err.api_code(), Some(code));
        assert_eq!(err.api_status(), Some(status));
        assert!(err.to_string().contains(message));
    }
}
