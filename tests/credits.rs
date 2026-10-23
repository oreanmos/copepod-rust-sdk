use copepod_sdk::{
    AiUsageReportRequest, CopepodClient, CopepodError, CreditCheckRequest, CreditReportRequest,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ME: &str = "/api/platform/orgs/org/apps/app/auth/users/me";

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
async fn credit_status_decodes_the_documented_shape() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/credits")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "period_start": "2026-10-01T00:00:00Z", "period_end": "2026-11-01T00:00:00Z",
            "allowance": 250, "allowance_used": 40,
            "topups": [{"grant_id": "g1", "remaining": 100, "expires_at": null}],
            "remaining_total": 310, "remaining_ratio": 0.88,
            "used_by_kind": {"ai": 30, "connector:ors": 10}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let s = client(&server)
        .get_current_credits("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(
        (s.allowance, s.allowance_used, s.remaining_total),
        (250, 40, 310)
    );
    assert_eq!(s.topups[0].grant_id, "g1");
    assert_eq!(s.topups[0].expires_at, None);
    assert_eq!(s.used_by_kind["connector:ors"], 10);
    assert!(
        s.connector_prices.is_empty(),
        "older servers omit the field"
    );
    let req = &server.received_requests().await.unwrap()[0];
    assert!(req.headers.get("x-api-key").is_none());
}

#[tokio::test]
async fn credit_status_decodes_connector_prices() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/credits")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "period_start": "2026-10-01T00:00:00Z", "period_end": "2026-11-01T00:00:00Z",
            "allowance": 250, "allowance_used": 0, "remaining_total": 250,
            "remaining_ratio": 1.0,
            "connector_prices": {"connector:ors": 5, "connector:weather": 1}
        })))
        .mount(&server)
        .await;
    let s = client(&server)
        .get_current_credits("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(s.connector_prices["connector:ors"], 5);
    assert_eq!(s.connector_prices["connector:weather"], 1);
}

#[tokio::test]
async fn check_allowed_and_402_exhausted() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/credits/check")))
        .and(body_json(json!({"kind": "ai"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"allowed": true, "remaining_total": 12})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/credits/check")))
        .and(body_json(
            json!({"kind": "connector:ors", "estimate_credits": 5}),
        ))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "status": 402, "code": "credits_exhausted", "error": "credits_exhausted",
            "message": "You're out of credits this month.",
            "remaining_total": 1, "resets_at": "2026-11-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let ok = c
        .check_current_credits(
            "org",
            "app",
            "users",
            &CreditCheckRequest {
                kind: "ai".into(),
                estimate_credits: None,
            },
        )
        .await
        .unwrap();
    assert!(ok.allowed);
    assert_eq!(ok.remaining_total, 12);
    let err = c
        .check_current_credits(
            "org",
            "app",
            "users",
            &CreditCheckRequest {
                kind: "connector:ors".into(),
                estimate_credits: Some(5),
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err.api_status(), Some(402));
    assert!(err.is_credits_exhausted());
    let d = err.credits_exhausted_details().unwrap();
    assert_eq!(
        (d.remaining_total, d.resets_at.as_str()),
        (1, "2026-11-01T00:00:00Z")
    );
}

#[tokio::test]
async fn report_charges_and_409_exhausted() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/credits/report")))
        .and(body_json(
            json!({"request_id": "r1", "kind": "ai", "cost_micro_eur": 25000}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"charged_credits": 3, "remaining_total": 7})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/credits/report")))
        .and(body_json(
            json!({"request_id": "r2", "kind": "connector:ors"}),
        ))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "credits_exhausted", "message": "out",
            "remaining_total": 1, "resets_at": "2026-11-01T00:00:00Z"})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let ok = c
        .report_current_credits(
            "org",
            "app",
            "users",
            &CreditReportRequest {
                request_id: "r1".into(),
                kind: "ai".into(),
                cost_micro_eur: Some(25_000),
                credits: None,
            },
        )
        .await
        .unwrap();
    assert_eq!((ok.charged_credits, ok.remaining_total), (3, 7));
    let err = c
        .report_current_credits(
            "org",
            "app",
            "users",
            &CreditReportRequest {
                request_id: "r2".into(),
                kind: "connector:ors".into(),
                cost_micro_eur: None,
                credits: None,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(err.api_status(), Some(409));
    assert!(err.is_credits_exhausted());
    let d = err.credits_exhausted_details().unwrap();
    assert_eq!(
        (d.remaining_total, d.resets_at.as_str()),
        (1, "2026-11-01T00:00:00Z")
    );
}

#[tokio::test]
async fn kind_is_trimmed_before_sending() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/credits/check")))
        .and(body_json(json!({"kind": "ai"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"allowed": true, "remaining_total": 1})),
        )
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .check_current_credits(
            "org",
            "app",
            "users",
            &CreditCheckRequest {
                kind: " ai ".into(),
                estimate_credits: None,
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn bad_kind_or_request_id_fails_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    let bad = CreditCheckRequest {
        kind: "connector:".into(),
        estimate_credits: None,
    };
    assert!(matches!(
        c.check_current_credits("org", "app", "users", &bad).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    let no_id = CreditReportRequest {
        request_id: "".into(),
        kind: "ai".into(),
        cost_micro_eur: Some(1),
        credits: None,
    };
    assert!(matches!(
        c.report_current_credits("org", "app", "users", &no_id)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn ai_usage_report_sends_kind_and_status_decodes_credit_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{ME}/ai-usage/report")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "event_id": "e1", "cost_micro_eur": 10000,
            "status": {"period_start": "a", "period_end": "b", "currency": "EUR",
                "budget_cents": 250, "used_cents": 1, "remaining_cents": 300,
                "remaining_ratio": 0.9, "allowed_models": [],
                "credit_remaining_total": 300,
                "credit_topups": [{"grant_id": "g", "remaining": 50, "expires_at": "2027-01-01T00:00:00Z"}]}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let out = client(&server)
        .report_current_ai_usage(
            "org",
            "app",
            "users",
            &AiUsageReportRequest {
                model: "m".into(),
                operation: "chat".into(),
                input_tokens: 1,
                output_tokens: 1,
                usage_unit: None,
                input_units: None,
                output_units: None,
                estimated: false,
                request_id: "r".into(),
                kind: Some("ai:chat".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(out.status.credit_remaining_total, Some(300));
    assert_eq!(out.status.credit_topups.unwrap()[0].remaining, 50);
    let req = &server.received_requests().await.unwrap()[0];
    let sent: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
    assert_eq!(sent["kind"], "ai:chat");
}
