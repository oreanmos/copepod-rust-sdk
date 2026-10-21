//! Laya `decide`, decide outcome and donations (AI4 S4): paths, methods, auth kind, bodies,
//! client-side limits (never stricter than copepod 8a6f6e74) and error mapping.

use copepod_sdk::error::{
    AI_PAUSED_FOR_REVIEW_CODE, LAYA_ADHOC_FORBIDDEN_CODE, LAYA_DECISION_UNKNOWN_CODE,
    LAYA_DONATION_CONSENT_REQUIRED_CODE, LAYA_DONATION_QUOTA_CODE, LAYA_INVALID_LABEL_CODE,
    LAYA_INVALID_REQUEST_CODE, LAYA_POLICY_VERSION_UNKNOWN_CODE, LAYA_TASK_NOT_DONATABLE_CODE,
    LAYA_TASK_UNKNOWN_CODE, LAYA_UNAVAILABLE_CODE, LAYA_UPGRADING_CODE,
};
use copepod_sdk::{
    CopepodClient, CopepodError, DecideOption, DecideOutcome, DecideQuestion, DecideRequest,
    Donation, DonationConsent, OutcomeSource, DONATION_POLICY_VERSION,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const U: &str = "/api/platform/orgs/org/apps/app/auth/users/me/ai";
const K: &str = "/api/platform/orgs/org/apps/app/ai";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("app-key")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn key_only(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-key")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn no_authorization(req: &Request) -> bool {
    !req.headers.contains_key("authorization")
}

fn no_api_key(req: &Request) -> bool {
    !req.headers.contains_key("x-api-key")
}

fn no_user_id(req: &Request) -> bool {
    !req.headers.contains_key("x-user-id")
}

fn labels(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn question(id: &str) -> DecideQuestion {
    DecideQuestion::choice(
        id,
        "Is it urgent?",
        vec![
            DecideOption::new("yes", "urgent"),
            DecideOption::new("no", "not urgent"),
        ],
    )
}

fn decide_body() -> Value {
    json!({
        "decision_id": "ld_01J",
        "task": "chat_guard",
        "task_version": 1,
        "mode": "active",
        "checkpoint": "2026-11-a",
        "answers": {"risk": {"choice": "jailbreak", "confidence": 0.93, "confident": true,
            "probabilities": {"normal": 0.05, "jailbreak": 0.93, "harmful": 0.02}}},
        "latency_ms": 301
    })
}

fn assert_api(error: CopepodError, status: u16, code: &str) {
    match error {
        CopepodError::Api {
            status: s,
            code: Some(c),
            ..
        } => {
            assert_eq!((s, c.as_str()), (status, code));
        }
        other => panic!("expected Api {status} {code}, got {other:?}"),
    }
}

async fn expect_no_request(server: &MockServer) {
    assert!(
        server.received_requests().await.unwrap().is_empty(),
        "validation must fail before any request"
    );
}

// ---- decide ----------------------------------------------------------------

#[tokio::test]
async fn decide_task_sends_bearer_only_and_decodes_confident() {
    let server = MockServer::start().await;
    let request = DecideRequest::task("chat_guard", json!({"text": "hi", "note_title": "t"}))
        .with_subject_ref("note:01J");
    Mock::given(method("POST"))
        .and(path(format!("{U}/decide")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .and(body_json(json!({
            "task": "chat_guard", "input": {"text": "hi", "note_title": "t"},
            "subject_ref": "note:01J"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(decide_body()))
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_decide("org", "app", "users", &request)
        .await
        .unwrap();
    assert_eq!(r.decision_id.as_deref(), Some("ld_01J"));
    assert_eq!(r.task.as_deref(), Some("chat_guard"));
    assert_eq!(r.task_version, Some(1));
    assert_eq!(r.mode, "active");
    assert_eq!(r.checkpoint.as_deref(), Some("2026-11-a"));
    assert_eq!(r.latency_ms, 301);
    let a = &r.answers["risk"];
    assert_eq!(a.choice, "jailbreak");
    assert!(a.confident && a.confidence > 0.92);
    assert_eq!(a.probabilities["normal"], 0.05);
}

#[tokio::test]
async fn decide_task_omits_absent_subject_ref() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/decide")))
        .and(body_json(
            json!({"task": "capture_route", "input": {"text": "milk"}}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"mode": "off", "answers": {}, "latency_ms": 0})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_decide(
            "org",
            "app",
            "users",
            &DecideRequest::task("capture_route", json!({"text": "milk"})),
        )
        .await
        .unwrap();
    assert_eq!(r.mode, "off");
    assert!(r.answers.is_empty() && r.decision_id.is_none() && r.task.is_none());
}

#[tokio::test]
async fn decide_decode_is_tolerant() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/decide")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "mode": "future_mode", "new_field": 1,
            "answers": {"q": {"choice": "a", "confidence": 0.5, "extra": true}}
        })))
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_decide("org", "app", "users", &DecideRequest::task("t", json!({})))
        .await
        .unwrap();
    assert_eq!(r.mode, "future_mode");
    let a = &r.answers["q"];
    assert!(!a.confident, "a missing `confident` is never confident");
    assert!(a.probabilities.is_empty());
    assert_eq!(r.latency_ms, 0);
}

#[tokio::test]
async fn bearer_decide_rejects_adhoc_before_any_request() {
    let server = MockServer::start().await;
    let request = DecideRequest::adhoc("state", vec![question("q")]);
    let error = client(&server)
        .ai_decide("org", "app", "users", &request)
        .await
        .unwrap_err();
    assert!(
        matches!(&error, CopepodError::InvalidArgument(m) if m.contains("laya_adhoc_forbidden")),
        "{error:?}"
    );
    expect_no_request(&server).await;
}

#[tokio::test]
async fn api_key_adhoc_sends_key_only_and_optional_user_id() {
    let server = MockServer::start().await;
    let request = DecideRequest::adhoc("The note says X", vec![question("urgent")]);
    let expected = json!({
        "state": "The note says X",
        "questions": [{"id": "urgent", "type": "choice", "question": "Is it urgent?",
            "options": [{"key": "yes", "description": "urgent"},
                        {"key": "no", "description": "not urgent"}]}]
    });
    let adhoc_response = json!({
        "task": null, "mode": "adhoc", "checkpoint": "2026-11-a", "latency_ms": 900,
        "answers": {"urgent": {"choice": "yes", "confidence": 0.8, "confident": false,
            "probabilities": {"yes": 0.8, "no": 0.2}}}
    });
    Mock::given(method("POST"))
        .and(path(format!("{K}/decide")))
        .and(header("x-api-key", "app-key"))
        .and(header("x-user-id", "user-7"))
        .and(no_authorization)
        .and(body_json(&expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(&adhoc_response))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{K}/decide")))
        .and(header("x-api-key", "app-key"))
        .and(no_authorization)
        .and(no_user_id)
        .and(body_json(&expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(&adhoc_response))
        .expect(1)
        .mount(&server)
        .await;
    // A client with only an API key (no bearer) is enough.
    let c = key_only(&server);
    let r = c
        .ai_decide_with_api_key("org", "app", &request, Some("user-7"))
        .await
        .unwrap();
    assert_eq!(r.mode, "adhoc");
    assert!(r.decision_id.is_none() && r.task.is_none() && r.task_version.is_none());
    assert!(!r.answers["urgent"].confident);
    c.ai_decide_with_api_key("org", "app", &request, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn api_key_task_decide_uses_the_key_route() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{K}/decide")))
        .and(header("x-api-key", "app-key"))
        .and(no_authorization)
        .and(body_json(
            json!({"task": "ticket_triage", "input": {"subject": "s", "body": "b"}}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(decide_body()))
        .expect(1)
        .mount(&server)
        .await;
    key_only(&server)
        .ai_decide_with_api_key(
            "org",
            "app",
            &DecideRequest::task("ticket_triage", json!({"subject": "s", "body": "b"})),
            None,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn api_key_decide_without_a_configured_key_fails_before_any_request() {
    let server = MockServer::start().await;
    let c = CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .auto_refresh(false)
        .build()
        .unwrap();
    let error = c
        .ai_decide_with_api_key("org", "app", &DecideRequest::task("t", json!({})), None)
        .await
        .unwrap_err();
    assert!(matches!(error, CopepodError::Auth(_)), "{error:?}");
    expect_no_request(&server).await;
}

#[tokio::test]
async fn adhoc_limits_match_the_server_and_are_not_stricter() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{K}/decide")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {"mode": "adhoc", "answers": {}, "latency_ms": 1}
        )))
        .expect(3)
        .mount(&server)
        .await;
    let c = key_only(&server);
    // Exactly 4,000 characters (multi-byte: counted as characters, not bytes), 3 questions, and
    // a whitespace-only state are all accepted by the server, so the SDK sends them.
    let state: String = "é".repeat(4_000);
    let three = vec![question("a"), question("b"), question("c")];
    c.ai_decide_with_api_key("org", "app", &DecideRequest::adhoc(state, three), None)
        .await
        .unwrap();
    c.ai_decide_with_api_key(
        "org",
        "app",
        &DecideRequest::adhoc("   ", vec![question("a")]),
        None,
    )
    .await
    .unwrap();
    // A caller may name an option-less or otherwise odd question: the server decides.
    let odd = DecideQuestion::choice("a", "q?", vec![]);
    c.ai_decide_with_api_key("org", "app", &DecideRequest::adhoc("s", vec![odd]), None)
        .await
        .unwrap();
}

#[tokio::test]
async fn adhoc_and_task_limits_reject_before_any_request() {
    let server = MockServer::start().await;
    let c = key_only(&server);
    let q = |n: usize| {
        (0..n)
            .map(|i| question(&format!("q{i}")))
            .collect::<Vec<_>>()
    };
    let mut not_choice = question("a");
    not_choice.kind = "free_text".into();
    let cases: Vec<(&str, DecideRequest)> = vec![
        ("4 questions", DecideRequest::adhoc("s", q(4))),
        ("0 questions", DecideRequest::adhoc("s", vec![])),
        ("empty state", DecideRequest::adhoc("", q(1))),
        ("4001 chars", DecideRequest::adhoc("é".repeat(4_001), q(1))),
        (
            "duplicate ids",
            DecideRequest::adhoc("s", vec![question("a"), question("a")]),
        ),
        (
            "non-choice type",
            DecideRequest::adhoc("s", vec![not_choice]),
        ),
        ("empty task", DecideRequest::task("", json!({}))),
    ];
    for (name, request) in cases {
        let error = c
            .ai_decide_with_api_key("org", "app", &request, None)
            .await
            .unwrap_err();
        assert!(
            matches!(error, CopepodError::InvalidArgument(_)),
            "{name}: {error:?}"
        );
    }
    // `subject_ref` length and the input shape are checked by the server, not here.
    expect_no_request(&server).await;
}

#[tokio::test]
async fn decide_rejects_bad_ids_and_user_id_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    let req = DecideRequest::task("t", json!({}));
    for (org, app, coll) in [
        ("", "app", "users"),
        ("o/x", "app", "users"),
        ("org", "app", ""),
    ] {
        assert!(matches!(
            c.ai_decide(org, app, coll, &req).await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    assert!(matches!(
        c.ai_decide_with_api_key("org", "app", &req, Some("")).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    expect_no_request(&server).await;
}

// ---- outcome ---------------------------------------------------------------

#[tokio::test]
async fn user_outcome_posts_bearer_and_records_source_user() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/decide/ld_01J/outcome")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .and(body_json(
            json!({"labels": {"risk": "jailbreak"}, "source": "user"}),
        ))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .ai_decide_outcome(
            "org",
            "app",
            "users",
            "ld_01J",
            &DecideOutcome::user(labels(&[("risk", "jailbreak")])),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn user_outcome_does_not_offer_tier2_or_tier3() {
    let server = MockServer::start().await;
    let c = client(&server);
    for source in [OutcomeSource::Tier2, OutcomeSource::Tier3] {
        let outcome = DecideOutcome::new(labels(&[("risk", "normal")]), source);
        let error = c
            .ai_decide_outcome("org", "app", "users", "ld_1", &outcome)
            .await
            .unwrap_err();
        assert!(
            matches!(&error, CopepodError::InvalidArgument(m) if m.contains("user")),
            "{error:?}"
        );
    }
    expect_no_request(&server).await;
}

#[tokio::test]
async fn key_outcome_sends_key_only_with_each_source() {
    let server = MockServer::start().await;
    for (source, wire) in [
        (OutcomeSource::Tier2, "tier2"),
        (OutcomeSource::Tier3, "tier3"),
        (OutcomeSource::User, "user"),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("{K}/decide/ld_9/outcome")))
            .and(header("x-api-key", "app-key"))
            .and(no_authorization)
            .and(body_json(
                json!({"labels": {"risk": "harmful"}, "source": wire}),
            ))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        key_only(&server)
            .ai_decide_outcome_with_api_key(
                "org",
                "app",
                "ld_9",
                &DecideOutcome::new(labels(&[("risk", "harmful")]), source),
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn outcome_validates_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    let ok = DecideOutcome::user(labels(&[("risk", "normal")]));
    let empty = DecideOutcome::user(BTreeMap::new());
    let blank = DecideOutcome::user(labels(&[("risk", "")]));
    assert!(matches!(
        c.ai_decide_outcome("org", "app", "users", "ld_1", &empty)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.ai_decide_outcome("org", "app", "users", "ld_1", &blank)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    for id in ["", "a/b", "ld 1"] {
        assert!(matches!(
            c.ai_decide_outcome("org", "app", "users", id, &ok).await,
            Err(CopepodError::InvalidArgument(_))
        ));
        assert!(matches!(
            c.ai_decide_outcome_with_api_key("org", "app", id, &ok)
                .await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    expect_no_request(&server).await;
}

// ---- errors ----------------------------------------------------------------

#[test]
fn error_code_constants_match_the_server() {
    assert_eq!(LAYA_INVALID_REQUEST_CODE, "laya_invalid_request");
    assert_eq!(LAYA_ADHOC_FORBIDDEN_CODE, "laya_adhoc_forbidden");
    assert_eq!(LAYA_TASK_UNKNOWN_CODE, "laya_task_unknown");
    assert_eq!(LAYA_DECISION_UNKNOWN_CODE, "laya_decision_unknown");
    assert_eq!(LAYA_INVALID_LABEL_CODE, "laya_invalid_label");
    assert_eq!(LAYA_UNAVAILABLE_CODE, "laya_unavailable");
    assert_eq!(LAYA_UPGRADING_CODE, "laya_upgrading");
    assert_eq!(AI_PAUSED_FOR_REVIEW_CODE, "ai_paused_for_review");
    assert_eq!(
        LAYA_POLICY_VERSION_UNKNOWN_CODE,
        "laya_policy_version_unknown"
    );
    assert_eq!(
        LAYA_DONATION_CONSENT_REQUIRED_CODE,
        "laya_donation_consent_required"
    );
    assert_eq!(LAYA_TASK_NOT_DONATABLE_CODE, "laya_task_not_donatable");
    assert_eq!(LAYA_DONATION_QUOTA_CODE, "laya_donation_quota");
    assert_eq!(DONATION_POLICY_VERSION, "2026-10");
}

async fn answer(server: &MockServer, verb: &str, route: &str, status: u16, code: &str) {
    Mock::given(method(verb))
        .and(path(route.to_string()))
        .respond_with(ResponseTemplate::new(status).set_body_json(json!(
            {"status": status, "code": code, "error": "x", "message": "m"}
        )))
        .mount(server)
        .await;
}

#[tokio::test]
async fn decide_and_outcome_errors_map_to_api_with_codes() {
    let server = MockServer::start().await;
    let req = DecideRequest::task("t", json!({}));
    let outcome = DecideOutcome::user(labels(&[("risk", "normal")]));
    let c = client(&server);

    answer(
        &server,
        "POST",
        &format!("{U}/decide"),
        404,
        LAYA_TASK_UNKNOWN_CODE,
    )
    .await;
    assert_api(
        c.ai_decide("org", "app", "users", &req).await.unwrap_err(),
        404,
        LAYA_TASK_UNKNOWN_CODE,
    );
    server.reset().await;
    answer(
        &server,
        "POST",
        &format!("{U}/decide"),
        503,
        LAYA_UNAVAILABLE_CODE,
    )
    .await;
    assert_api(
        c.ai_decide("org", "app", "users", &req).await.unwrap_err(),
        503,
        LAYA_UNAVAILABLE_CODE,
    );
    server.reset().await;
    answer(
        &server,
        "POST",
        &format!("{U}/decide"),
        400,
        LAYA_INVALID_REQUEST_CODE,
    )
    .await;
    assert_api(
        c.ai_decide("org", "app", "users", &req).await.unwrap_err(),
        400,
        LAYA_INVALID_REQUEST_CODE,
    );
    server.reset().await;
    answer(
        &server,
        "POST",
        &format!("{K}/decide"),
        403,
        LAYA_ADHOC_FORBIDDEN_CODE,
    )
    .await;
    assert_api(
        c.ai_decide_with_api_key("org", "app", &req, None)
            .await
            .unwrap_err(),
        403,
        LAYA_ADHOC_FORBIDDEN_CODE,
    );
    server.reset().await;
    answer(
        &server,
        "POST",
        &format!("{U}/decide"),
        403,
        AI_PAUSED_FOR_REVIEW_CODE,
    )
    .await;
    assert_api(
        c.ai_decide("org", "app", "users", &req).await.unwrap_err(),
        403,
        AI_PAUSED_FOR_REVIEW_CODE,
    );
    server.reset().await;

    let route = format!("{U}/decide/ld_1/outcome");
    for (status, code) in [
        (404, LAYA_DECISION_UNKNOWN_CODE),
        (400, LAYA_INVALID_LABEL_CODE),
        (503, LAYA_UPGRADING_CODE),
    ] {
        answer(&server, "POST", &route, status, code).await;
        assert_api(
            c.ai_decide_outcome("org", "app", "users", "ld_1", &outcome)
                .await
                .unwrap_err(),
            status,
            code,
        );
        server.reset().await;
    }
}

#[tokio::test]
async fn a_node_without_the_routes_is_a_plain_404_without_a_code() {
    let server = MockServer::start().await;
    // No mock mounted: wiremock answers an empty 404, like axum on an older node.
    let c = client(&server);
    let error = c
        .ai_decide("org", "app", "users", &DecideRequest::task("t", json!({})))
        .await
        .unwrap_err();
    assert!(
        matches!(
            error,
            CopepodError::Api {
                status: 404,
                code: None,
                ..
            }
        ),
        "{error:?}"
    );
    let error = c
        .ai_donations_status("org", "app", "users")
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        CopepodError::Api {
            status: 404,
            code: None,
            ..
        }
    ));
}

// ---- donations -------------------------------------------------------------

#[tokio::test]
async fn donations_status_gets_with_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{U}/donations")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "enabled": true, "count": 4, "oldest": "2026-10-01T00:00:00Z",
            "policy_version": "2026-10", "future": 1
        })))
        .expect(1)
        .mount(&server)
        .await;
    let s = client(&server)
        .ai_donations_status("org", "app", "users")
        .await
        .unwrap();
    assert!(s.enabled);
    assert_eq!(s.count, 4);
    assert_eq!(s.oldest.as_deref(), Some("2026-10-01T00:00:00Z"));
    assert_eq!(s.policy_version, "2026-10");
}

#[tokio::test]
async fn donations_status_decodes_an_empty_account() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{U}/donations")))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"enabled": false, "count": 0, "oldest": null, "policy_version": "2026-10"}),
        ))
        .mount(&server)
        .await;
    let s = client(&server)
        .ai_donations_status("org", "app", "users")
        .await
        .unwrap();
    assert!(!s.enabled && s.count == 0 && s.oldest.is_none());
}

#[tokio::test]
async fn consent_grant_and_revoke_put_the_expected_bodies() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("{U}/donations/consent")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(
            json!({"enabled": true, "policy_version": "2026-10"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"enabled": true, "consent_id": "dc_1", "granted_at": "2026-10-21T10:00:00Z"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(format!("{U}/donations/consent")))
        .and(body_json(json!({"enabled": false})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"enabled": false, "deleted": 7})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let granted = c
        .ai_set_donation_consent("org", "app", "users", &DonationConsent::grant())
        .await
        .unwrap();
    assert!(granted.enabled);
    assert_eq!(granted.consent_id.as_deref(), Some("dc_1"));
    assert_eq!(granted.granted_at.as_deref(), Some("2026-10-21T10:00:00Z"));
    assert_eq!(granted.deleted, None);
    let revoked = c
        .ai_set_donation_consent("org", "app", "users", &DonationConsent::revoke())
        .await
        .unwrap();
    assert!(!revoked.enabled);
    assert_eq!(revoked.deleted, Some(7));
    assert!(revoked.consent_id.is_none());
}

#[tokio::test]
async fn consent_grant_needs_a_policy_version_and_unknown_one_maps_the_server_error() {
    let server = MockServer::start().await;
    let c = client(&server);
    let missing = DonationConsent {
        enabled: true,
        policy_version: None,
    };
    assert!(matches!(
        c.ai_set_donation_consent("org", "app", "users", &missing)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    expect_no_request(&server).await;
    // A version the SDK does not know is the server's call (it may know newer ones).
    answer(
        &server,
        "PUT",
        &format!("{U}/donations/consent"),
        400,
        LAYA_POLICY_VERSION_UNKNOWN_CODE,
    )
    .await;
    let stale = DonationConsent {
        enabled: true,
        policy_version: Some("2020-01".into()),
    };
    assert_api(
        c.ai_set_donation_consent("org", "app", "users", &stale)
            .await
            .unwrap_err(),
        400,
        LAYA_POLICY_VERSION_UNKNOWN_CODE,
    );
}

#[tokio::test]
async fn donate_posts_text_labels_and_optional_language() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/donations")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .and(body_json(json!({
            "task": "capture_route", "text": "milk and eggs",
            "labels": {"module": "shopping"}, "language": "en"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": "le_1"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/donations")))
        .and(body_json(json!({
            "task": "capture_route", "text": "x", "labels": {"module": "notes"}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": "le_2"})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let mut d = Donation::new(
        "capture_route",
        "milk and eggs",
        labels(&[("module", "shopping")]),
    );
    d.language = Some("en".into());
    assert_eq!(
        c.ai_donate("org", "app", "users", &d).await.unwrap().id,
        "le_1"
    );
    let d = Donation::new("capture_route", "x", labels(&[("module", "notes")]));
    assert_eq!(
        c.ai_donate("org", "app", "users", &d).await.unwrap().id,
        "le_2"
    );
}

#[tokio::test]
async fn donate_validates_before_any_request_and_matches_the_server_limits() {
    let server = MockServer::start().await;
    let c = client(&server);
    let l = labels(&[("module", "shopping")]);
    let mut long_tag = Donation::new("t", "x", l.clone());
    long_tag.language = Some("x".repeat(17));
    let mut empty_tag = Donation::new("t", "x", l.clone());
    empty_tag.language = Some(String::new());
    let bad = [
        Donation::new("", "x", l.clone()),
        Donation::new("t", "", l.clone()),
        Donation::new("t", "  \n", l.clone()),
        Donation::new("t", "é".repeat(4_001), l.clone()),
        Donation::new("t", "x", BTreeMap::new()),
        Donation::new("t", "x", labels(&[("module", "")])),
        long_tag,
        empty_tag,
    ];
    for d in &bad {
        assert!(
            matches!(
                c.ai_donate("org", "app", "users", d).await,
                Err(CopepodError::InvalidArgument(_))
            ),
            "{d:?}"
        );
    }
    expect_no_request(&server).await;
    // 4,000 characters (not bytes) and a 16-byte language tag are accepted.
    Mock::given(method("POST"))
        .and(path(format!("{U}/donations")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": "le"})))
        .expect(1)
        .mount(&server)
        .await;
    let mut edge = Donation::new("t", "é".repeat(4_000), l);
    edge.language = Some("x".repeat(16));
    c.ai_donate("org", "app", "users", &edge).await.unwrap();
}

#[tokio::test]
async fn donate_errors_map_to_api_with_codes() {
    let server = MockServer::start().await;
    let c = client(&server);
    let d = Donation::new("capture_route", "x", labels(&[("module", "notes")]));
    for (status, code) in [
        (403, LAYA_DONATION_CONSENT_REQUIRED_CODE),
        (400, LAYA_TASK_NOT_DONATABLE_CODE),
        (400, LAYA_INVALID_LABEL_CODE),
        (429, LAYA_DONATION_QUOTA_CODE),
        (503, LAYA_UPGRADING_CODE),
    ] {
        answer(&server, "POST", &format!("{U}/donations"), status, code).await;
        assert_api(
            c.ai_donate("org", "app", "users", &d).await.unwrap_err(),
            status,
            code,
        );
        server.reset().await;
    }
}

#[tokio::test]
async fn delete_donations_returns_the_count() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("{U}/donations")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted": 12})))
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_delete_donations("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(r.deleted, 12);
}

#[tokio::test]
async fn donation_methods_reject_empty_ids() {
    let server = MockServer::start().await;
    let c = client(&server);
    assert!(c.ai_donations_status("", "app", "users").await.is_err());
    assert!(c.ai_delete_donations("org", "app", "").await.is_err());
    assert!(c
        .ai_set_donation_consent("org", "", "users", &DonationConsent::revoke())
        .await
        .is_err());
    expect_no_request(&server).await;
}

// ---- scoped ----------------------------------------------------------------

#[tokio::test]
async fn scoped_ai_wraps_decide_outcome_and_donations() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/decide")))
        .respond_with(ResponseTemplate::new(200).set_body_json(decide_body()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/decide/ld_01J/outcome")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{U}/donations")))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"enabled": false, "count": 0, "oldest": null, "policy_version": "2026-10"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{K}/decide")))
        .and(header("x-api-key", "app-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(decide_body()))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let ai = c.app("org", "app").ai("users");
    ai.decide(&DecideRequest::task("chat_guard", json!({"text": "hi"})))
        .await
        .unwrap();
    ai.decide_outcome(
        "ld_01J",
        &DecideOutcome::user(labels(&[("risk", "normal")])),
    )
    .await
    .unwrap();
    assert!(!ai.donations_status().await.unwrap().enabled);
    ai.decide_with_api_key(&DecideRequest::task("chat_guard", json!({})), None)
        .await
        .unwrap();
}
