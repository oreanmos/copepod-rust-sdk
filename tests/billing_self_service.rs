use copepod_sdk::{
    AppBillingCatalog, AppBillingSettings, BillingEvidence, BillingIntentCreate, CopepodClient,
    CopepodError, MeCheckoutRequest, MeTrialStartRequest,
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
