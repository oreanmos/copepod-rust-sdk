use copepod_sdk::{
    AppBillingCatalog, AppBillingSettings, AppRegisterRequest, BillingEvidence,
    BillingIntentCreate, CopepodClient, CopepodError, MeCheckoutRequest, MePaymentRecoveryRequest,
    MeTrialStartRequest,
};
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

fn trial_body() -> MeTrialStartRequest {
    MeTrialStartRequest {
        billing_country: Some("GB".into()),
        evidence: Some(BillingEvidence {
            ip_country: Some("GB".into()),
        }),
    }
}

fn checkout_body(promo_code: Option<&str>) -> MeCheckoutRequest {
    MeCheckoutRequest {
        plan_slug: "pro".into(),
        billing_interval: Some("month".into()),
        billing_country: Some("GB".into()),
        evidence: Some(BillingEvidence {
            ip_country: Some("GB".into()),
        }),
        promo_code: promo_code.map(str::to_owned),
        redirect_url: Some("https://oikonotes.com/billing/return".into()),
    }
}

#[tokio::test]
async fn me_start_trial_posts_country_with_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/trial",
        ))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(json!({
            "billing_country": "GB",
            "evidence": {"ip_country": "GB"}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "trial": {"plan_slug": "pro", "trial_ends_at": "2026-10-18T00:00:00Z"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)
        .me_start_trial("org", "app", "users", &trial_body())
        .await
        .unwrap();

    assert_eq!(result.trial.plan_slug, "pro");
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("x-api-key"));
}

#[tokio::test]
async fn me_start_trial_maps_trial_already_used_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/trial",
        ))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "trial_already_used",
            "message": "A trial was already used"
        })))
        .expect(1)
        .mount(&server)
        .await;

    assert!(matches!(
        client(&server)
            .me_start_trial("org", "app", "users", &trial_body())
            .await,
        Err(CopepodError::Api {
            status: 409,
            code: Some(code),
            ..
        }) if code == "trial_already_used"
    ));
}

#[tokio::test]
async fn me_start_checkout_sends_idempotency_key_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/checkout",
        ))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "checkout-1"))
        .and(body_json(json!({
            "plan_slug": "pro",
            "billing_interval": "month",
            "billing_country": "GB",
            "evidence": {"ip_country": "GB"},
            "promo_code": "BETA50",
            "redirect_url": "https://oikonotes.com/billing/return"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "payment_id": "payment-1",
            "checkout_url": "https://checkout.example.invalid/1",
            "period_starts_at": "2026-11-04T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client(&server)
        .me_start_checkout(
            "org",
            "app",
            "users",
            "checkout-1",
            &checkout_body(Some("BETA50")),
        )
        .await
        .unwrap();

    assert_eq!(result.payment_id, "payment-1");
    assert!(result
        .checkout_url
        .starts_with("https://checkout.example.invalid/"));
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("x-api-key"));
}

#[tokio::test]
async fn me_start_checkout_maps_discount_code_invalid() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/checkout",
        ))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "code": "discount_code_invalid",
            "message": "Discount code is invalid"
        })))
        .expect(1)
        .mount(&server)
        .await;

    assert!(matches!(
        client(&server)
            .me_start_checkout("org", "app", "users", "checkout-1", &checkout_body(None))
            .await,
        Err(CopepodError::Api {
            status: 422,
            code: Some(code),
            ..
        }) if code == "discount_code_invalid"
    ));
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert!(body.get("promo_code").is_none());
}

#[tokio::test]
async fn me_start_checkout_rejects_bad_key_offline() {
    let server = MockServer::start().await;
    for key in ["", "bad key", &"a".repeat(161)] {
        assert!(matches!(
            client(&server)
                .me_start_checkout("org", "app", "users", key, &checkout_body(None))
                .await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn me_start_checkout_never_sends_api_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/checkout",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "payment_id": "p",
            "checkout_url": "https://checkout.example.invalid/p",
            "period_starts_at": "2026-11-04T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .me_start_checkout("org", "app", "users", "checkout-1", &checkout_body(None))
        .await
        .unwrap();
    for request in server.received_requests().await.unwrap() {
        assert!(!request.headers.contains_key("x-api-key"));
    }
}

#[test]
fn intent_create_serialises_new_fields_only_when_set() {
    let base = BillingIntentCreate {
        email: "test@example.invalid".into(),
        plan_slug: "pro".into(),
        ..Default::default()
    };
    let empty = serde_json::to_value(&base).unwrap();
    assert!(empty.get("billing_interval").is_none());
    assert!(empty.get("billing_country").is_none());
    assert!(empty.get("evidence").is_none());

    let enriched = BillingIntentCreate {
        billing_interval: Some("year".into()),
        billing_country: Some("GB".into()),
        evidence: Some(BillingEvidence {
            ip_country: Some("GB".into()),
        }),
        ..base
    };
    let value = serde_json::to_value(enriched).unwrap();
    assert_eq!(value["billing_interval"], "year");
    assert_eq!(value["billing_country"], "GB");
    assert_eq!(value["evidence"]["ip_country"], "GB");
}

#[test]
fn catalog_parses_signup_policy() {
    let catalog: AppBillingCatalog = serde_json::from_value(json!({
        "plans": [],
        "addons": [],
        "signup": {
            "registration_requires_billing": true,
            "allowed_billing_countries": ["GB"],
            "trial_requires_payment_method": false,
            "trial_plan_slug": "pro",
            "trial_duration_days": 14
        }
    }))
    .unwrap();
    let signup = catalog.signup.unwrap();
    assert_eq!(signup.allowed_billing_countries, ["GB"]);
    assert!(!signup.trial_requires_payment_method);
    assert_eq!(signup.trial_duration_days, Some(14));
}

#[test]
fn settings_default_trial_requires_payment_method_true() {
    let settings: AppBillingSettings = serde_json::from_value(json!({})).unwrap();
    assert!(settings.trial_requires_payment_method);
    let serialized: Value = serde_json::to_value(settings).unwrap();
    assert_eq!(serialized["trial_requires_payment_method"], true);
}

#[tokio::test]
async fn me_start_checkout_rejects_blank_redirect_offline() {
    let server = MockServer::start().await;
    let mut body = checkout_body(None);
    body.redirect_url = Some("  ".into());
    assert!(matches!(
        client(&server)
            .me_start_checkout("org", "app", "users", "checkout-1", &body)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn app_register_returns_record_and_trial_from_the_server_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org1/apps/app1/auth/users/register",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "t",
            "refresh_token": "r",
            "record": {"id": "u1", "email": "a@b.co"},
            "trial": {"plan_slug": "pro", "trial_ends_at": "2026-10-19T00:00:00Z"}
        })))
        .mount(&server)
        .await;
    let body = AppRegisterRequest {
        email: "a@b.co".into(),
        password: "secret-password".into(),
        name: None,
        billing_country: Some("GB".into()),
        evidence: None,
    };
    let resp = client(&server)
        .app_register("org1", "app1", "users", &body)
        .await
        .unwrap();
    assert_eq!(resp.token, "t");
    assert_eq!(resp.record["id"], "u1");
    assert_eq!(resp.trial.unwrap().plan_slug, "pro");
}

#[tokio::test]
async fn app_register_without_trial_deserializes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org1/apps/app1/auth/users/register",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "t", "refresh_token": "r", "record": {"id": "u1"}, "trial": null
        })))
        .mount(&server)
        .await;
    let resp = client(&server)
        .app_register("org1", "app1", "users", &json!({"email": "a@b.co"}))
        .await
        .unwrap();
    assert!(resp.trial.is_none());
}

#[tokio::test]
async fn me_start_checkout_surfaces_checkout_pending_details() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/checkout",
        ))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "status": 409,
            "code": "checkout_pending",
            "error": "checkout_pending",
            "message": "an earlier checkout is still open; complete it or wait for it to expire",
            "checkout_url": "https://checkout.example.invalid/open",
            "expires_at": "2026-10-05T10:14:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let err = client(&server)
        .me_start_checkout("org", "app", "users", "checkout-1", &checkout_body(None))
        .await
        .unwrap_err();

    // Matching on status and code keeps working for either error shape.
    assert_eq!(err.api_status(), Some(409));
    assert_eq!(err.api_code(), Some("checkout_pending"));
    let details = err.checkout_pending_details().expect("typed details");
    assert_eq!(
        details.checkout_url.as_deref(),
        Some("https://checkout.example.invalid/open")
    );
    assert_eq!(details.expires_at, "2026-10-05T10:14:00Z");
}

#[tokio::test]
async fn me_start_checkout_pending_without_url_has_none_checkout_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/users/me/subscription/checkout",
        ))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "status": 409, "code": "checkout_pending", "error": "checkout_pending",
            "message": "open", "checkout_url": null, "expires_at": "2026-10-05T10:14:00Z"
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .me_start_checkout("org", "app", "users", "checkout-1", &checkout_body(None))
        .await
        .unwrap_err();
    assert_eq!(err.checkout_pending_details().unwrap().checkout_url, None);
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
    let err = client(&server)
        .me_get_receipt_html("org", "app", "users", "")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
}

#[tokio::test]
async fn me_get_receipt_html_maps_receipt_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/receipts/nope")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "receipt not found", "code": "receipt_not_found"
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .me_get_receipt_html("org", "app", "users", "nope")
        .await
        .unwrap_err();
    assert_eq!(err.api_code(), Some("receipt_not_found"));
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
            "error": "not past due", "code": "not_past_due"
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .me_start_payment_recovery("org", "app", "users", "rec-1", &recovery_body())
        .await
        .unwrap_err();
    assert_eq!(err.api_code(), Some("not_past_due"));
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
