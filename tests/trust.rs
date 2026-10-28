use copepod_sdk::*;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const APP: &str = "/api/platform/orgs/org/apps/app";
const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// Bearer and API key both set: a key route must send only the key, a bearer route only the bearer.
fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("key-1")
        .auto_refresh(false)
        .build()
        .unwrap()
}

async fn expect_key(server: &MockServer) {
    let r = &server.received_requests().await.unwrap()[0];
    assert_eq!(r.headers.get("x-api-key").unwrap(), "key-1");
    assert!(
        r.headers.get("authorization").is_none(),
        "key route sent Authorization"
    );
}

async fn expect_bearer(server: &MockServer) {
    let r = &server.received_requests().await.unwrap()[0];
    assert!(
        r.headers.get("x-api-key").is_none(),
        "bearer route sent X-API-Key"
    );
}

fn moderate_body() -> serde_json::Value {
    json!({"decision":"review","reasons":["harm:hate_and_discrimination"],"engine":"laya",
           "confidence":0.7,"decision_id":"td_1","degraded":false,
           "restriction":{"scope":"public_posting","until":"2026-11-01T00:00:00Z"}})
}

fn check() -> ModerateRequest {
    let mut r = ModerateRequest::new(ModerationPurpose::Comment, "blog_comment:abc", "hello");
    r.actor.user_id = Some("u1".into());
    r.reviewer = ModerationReviewer::Operator;
    r
}

async fn mount(server: &MockServer, m: &str, p: String, status: u16, body: serde_json::Value) {
    Mock::given(method(m))
        .and(path(p))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn moderate_posts_with_key_only_and_decodes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/ai/moderate")))
        .and(header("x-api-key", "key-1"))
        .and(body_json(
            json!({"purpose":"comment","subject_ref":"blog_comment:abc","text":"hello",
            "context":{"check_harm":false},"actor":{"user_id":"u1"},"reviewer":"operator"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderate_body()))
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .moderate("org", "app", &check())
        .await
        .unwrap();
    assert_eq!(r.decision, ModerationDecision::Review);
    assert_eq!(r.engine, ModerationEngine::Laya);
    assert_eq!(r.decision_id, "td_1");
    assert_eq!(
        r.restriction.unwrap().scope,
        RestrictionScope::PublicPosting
    );
    expect_key(&server).await;
}

#[tokio::test]
async fn moderate_as_user_uses_bearer_and_scoped_wrapper_matches() {
    let server = MockServer::start().await;
    mount(
        &server,
        "POST",
        format!("{APP}/auth/users/me/ai/moderate"),
        200,
        moderate_body(),
    )
    .await;
    let r = client(&server)
        .app("org", "app")
        .trust()
        .moderate_as_user("users", &check())
        .await
        .unwrap();
    assert_eq!(r.confidence, 0.7);
    let req = &server.received_requests().await.unwrap()[0];
    assert_eq!(
        req.headers.get("authorization").unwrap(),
        "Bearer user-bearer"
    );
    expect_bearer(&server).await;
}

#[tokio::test]
async fn record_outcome_posts_with_key_only() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/trust/decisions/td_1/outcome")))
        .and(body_json(
            json!({"outcome":"removed","wrong_decision":true,"note":"spam"}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"decision_id":"td_1","outcome":"removed","wrong_decision":true}),
            ),
        )
        .expect(1)
        .mount(&server)
        .await;
    let req = ModerationOutcomeRequest {
        outcome: ModerationOutcome::Removed,
        wrong_decision: true,
        note: Some("spam".into()),
    };
    let r = client(&server)
        .record_moderation_outcome("org", "app", "td_1", &req)
        .await
        .unwrap();
    assert_eq!(r.outcome, ModerationOutcome::Removed);
    expect_key(&server).await;
}

fn report() -> ContentReportRequest {
    ContentReportRequest {
        subject_ref: "blog_comment:abc".into(),
        subject_url: None,
        category: ReportCategory::Spam,
        details: "bad".into(),
        reporter_email: None,
        reporter_ip_hash: HASH.into(),
        subject_owner_user_id: Some("u2".into()),
    }
}

#[tokio::test]
async fn submit_report_posts_with_key_only_and_maps_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/reports")))
        .and(body_json(
            json!({"subject_ref":"blog_comment:abc","category":"spam","details":"bad",
            "reporter_ip_hash":HASH,"subject_owner_user_id":"u2"}),
        ))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "report_id":"cr_1","status":"received","priority":"routine","duplicate":false})))
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .submit_content_report("org", "app", &report())
        .await
        .unwrap();
    assert_eq!(
        (r.report_id.as_str(), r.priority),
        ("cr_1", ReportPriority::Routine)
    );
    expect_key(&server).await;

    server.reset().await;
    mount(&server, "POST", format!("{APP}/reports"), 429,
        json!({"status":429,"code":"report_rate_limited","error":"report_rate_limited","message":"Too many reports."})).await;
    let err = client(&server)
        .submit_content_report("org", "app", &report())
        .await
        .unwrap_err();
    assert!(err.is_report_rate_limited());
    assert_eq!(err.api_status(), Some(429));
    assert_eq!(err.api_code(), Some(REPORT_RATE_LIMITED_CODE));
}

#[tokio::test]
async fn trust_actions_feed_sends_cursor_with_key_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/trust/actions")))
        .and(query_param("after", "7"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "actions":[{"seq":8,"kind":"takedown","subject_ref":"blog_comment:abc","user_id":null,"created":"t"},
                       {"seq":9,"kind":"from_the_future","subject_ref":null,"user_id":"u1","created":"t"}],
            "next_after":9})))
        .expect(1)
        .mount(&server)
        .await;
    let page = client(&server)
        .list_trust_actions("org", "app", Some(7), 50)
        .await
        .unwrap();
    assert_eq!(page.next_after, 9);
    assert_eq!(page.actions[0].kind, TrustActionKind::Takedown);
    assert_eq!(page.actions[1].kind, TrustActionKind::Unknown);
    expect_key(&server).await;
}

#[tokio::test]
async fn abuse_signal_posts_with_key_only() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/trust/signals")))
        .and(body_json(json!({"user_id":"u1","kind":"chat_jailbreak","evidence_ref":"td_1"})))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "signal_id":"as_1",
            "restriction":{"scope":"hosted_ai","until":"2026-11-02T00:00:00Z","reason_code":"abuse_score"}})))
        .expect(1)
        .mount(&server)
        .await;
    let req = AbuseSignalRequest {
        user_id: "u1".into(),
        kind: AbuseSignalKind::ChatJailbreak,
        evidence_ref: "td_1".into(),
    };
    let r = client(&server)
        .record_abuse_signal("org", "app", &req)
        .await
        .unwrap();
    assert_eq!(r.restriction.unwrap().scope, RestrictionScope::HostedAi);
    expect_key(&server).await;
}

#[tokio::test]
async fn my_restrictions_uses_bearer_and_maps_suspension() {
    let server = MockServer::start().await;
    mount(
        &server,
        "GET",
        format!("{APP}/auth/users/me/restrictions"),
        200,
        json!({
        "restrictions":[{"scope":"account","until":null,"reason_code":"owner"}],"suspended":true}),
    )
    .await;
    let r = client(&server)
        .my_restrictions("org", "app", "users")
        .await
        .unwrap();
    assert!(r.suspended);
    assert_eq!(r.restrictions[0].until, None);
    expect_bearer(&server).await;

    server.reset().await;
    mount(&server, "GET", format!("{APP}/auth/users/me/restrictions"), 403,
        json!({"status":403,"code":"account_suspended","error":"account_suspended","message":"suspended"})).await;
    let err = client(&server)
        .my_restrictions("org", "app", "users")
        .await
        .unwrap_err();
    assert!(err.is_account_suspended());
    assert_eq!(err.api_code(), Some(ACCOUNT_SUSPENDED_CODE));
}

#[tokio::test]
async fn ai_paused_for_review_carries_scope_and_until() {
    let server = MockServer::start().await;
    mount(
        &server,
        "POST",
        format!("{APP}/auth/users/me/ai/moderate"),
        403,
        json!({
        "status":403,"code":"ai_paused_for_review","error":"ai_paused_for_review",
        "message":"paused","scope":"hosted_ai","until":"2026-11-02T00:00:00Z"}),
    )
    .await;
    let err = client(&server)
        .moderate_as_user("org", "app", "users", &check())
        .await
        .unwrap_err();
    assert!(err.is_ai_paused_for_review());
    let d = err.ai_paused_for_review_details().unwrap();
    assert_eq!(d.scope, RestrictionScope::HostedAi);
    assert_eq!(d.until.as_deref(), Some("2026-11-02T00:00:00Z"));
}

#[tokio::test]
async fn content_rejected_maps_on_record_edit() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(format!("{APP}/records/posts/r1")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "status":422,"code":"content_rejected","error":"content_rejected","message":"rejected"})))
        .expect(1)
        .mount(&server)
        .await;
    let err = client(&server)
        .update_record("org", "app", "posts", "r1", &json!({"body":"x"}))
        .await
        .unwrap_err();
    assert!(err.is_content_rejected());
    assert_eq!(err.api_status(), Some(422));
    assert_eq!(err.api_code(), Some(CONTENT_REJECTED_CODE));
}

#[tokio::test]
async fn review_queue_list_sends_filters_with_bearer_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/trust/queue")))
        .and(query_param("status", "open"))
        .and(query_param("kind", "report"))
        .and(query_param("limit", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items":[{"id":"cr_1","kind":"report","purpose":"report","subject_ref":"blog_comment:abc",
                      "reasons":[],"priority":"urgent","created":"t","due_at":"t2","status":"open"}],
            "next_cursor":null})))
        .expect(1)
        .mount(&server)
        .await;
    let q = ReviewQueueQuery {
        status: Some(ReviewQueueStatus::Open),
        kind: Some(ReviewItemKind::Report),
        limit: Some(20),
        cursor: None,
    };
    let page = client(&server)
        .list_review_queue("org", "app", &q)
        .await
        .unwrap();
    assert_eq!(page.items[0].priority, Some(ReportPriority::Urgent));
    expect_bearer(&server).await;
}

#[tokio::test]
async fn resolve_item_posts_action() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/trust/queue/td_1/resolve")))
        .and(body_json(json!({"action":"remove","wrong_decision":false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"id":"td_1","status":"resolved","outcome":"removed","wrong_decision":false}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let req = ResolveReviewRequest {
        action: ReviewAction::Remove,
        wrong_decision: false,
        note: None,
    };
    let r = client(&server)
        .resolve_review_item("org", "app", "td_1", &req)
        .await
        .unwrap();
    assert_eq!(r.status, ReviewQueueStatus::Resolved);
    expect_bearer(&server).await;
}

#[tokio::test]
async fn decisions_list_sends_filters_and_decodes_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/trust/decisions")))
        .and(query_param("purpose", "comment"))
        .and(query_param("decision", "block"))
        .and(query_param("degraded", "true"))
        .and(query_param("before", "t~td_9"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items":[{"id":"td_1","purpose":"comment","subject_ref":"blog_comment:abc","decision":"block",
              "reasons":[],"engine":"tier0","confidence":1.0,"degraded":true,"reviewer":"app",
              "review_state":"none","outcome":null,"wrong_decision":true,"wrong_by":"app","created":"t"}],
            "next_cursor":null,"total":1,"degraded":1})))
        .expect(1)
        .mount(&server)
        .await;
    let q = TrustDecisionQuery {
        purpose: Some(ModerationPurpose::Comment),
        decision: Some(ModerationDecision::Block),
        degraded_only: true,
        before: Some("t~td_9".into()),
        ..Default::default()
    };
    let page = client(&server)
        .list_trust_decisions("org", "app", &q)
        .await
        .unwrap();
    assert_eq!((page.total, page.degraded), (1, 1));
    assert_eq!(page.items[0].wrong_by.as_deref(), Some("app"));
    expect_bearer(&server).await;
}

#[tokio::test]
async fn wrong_decision_and_reports_list() {
    let server = MockServer::start().await;
    mount(
        &server,
        "POST",
        format!("{APP}/trust/decisions/td_1/wrong"),
        200,
        json!({"decision_id":"td_1","outcome":"approved","wrong_decision":true,"action":"restore"}),
    )
    .await;
    let r = client(&server)
        .mark_decision_wrong("org", "app", "td_1", &WrongDecisionRequest::default())
        .await
        .unwrap();
    assert_eq!(r.action.as_deref(), Some("restore"));

    server.reset().await;
    Mock::given(method("GET")).and(path(format!("{APP}/trust/reports")))
        .and(query_param("status", "resolved"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items":[{"id":"cr_1","subject_ref":"blog_comment:abc","category":"hate","details":"d",
              "priority":"urgent","status":"actioned","created":"t","due_at":"t","resolved_at":"t"}],
            "next_cursor":null}))).expect(1).mount(&server).await;
    let page = client(&server)
        .list_content_reports(
            "org",
            "app",
            &ContentReportQuery {
                resolved: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(page.items[0].category, ReportCategory::Hate);
}

#[tokio::test]
async fn user_trust_flagged_users_and_lift() {
    let server = MockServer::start().await;
    let trust = json!({"scores":{"hosted_ai":1.5,"public_posting":0.0,"invites":0.0},
        "signals":[{"kind":"chat_harm","weight":1.5,"evidence_ref":"td_1","created":"t"}],
        "restrictions":[{"scope":"hosted_ai","source":"auto","reason":"r","created":"t","until":"u"}],
        "suspended":null});
    mount(
        &server,
        "GET",
        format!("{APP}/app-users/u1/trust"),
        200,
        trust.clone(),
    )
    .await;
    let t = client(&server)
        .get_app_user_trust("org", "app", "u1")
        .await
        .unwrap();
    assert_eq!(t.scores.hosted_ai, 1.5);
    assert_eq!(t.signals[0].kind, AbuseSignalKind::ChatHarm);
    assert!(t.suspended.is_none());

    server.reset().await;
    let mut flagged = trust;
    flagged["user_id"] = json!("u1");
    mount(
        &server,
        "GET",
        format!("{APP}/trust/users"),
        200,
        json!({"users":[flagged]}),
    )
    .await;
    let u = client(&server)
        .list_flagged_app_users("org", "app")
        .await
        .unwrap();
    assert_eq!(u.users[0].user_id, "u1");

    server.reset().await;
    mount(
        &server,
        "POST",
        format!("{APP}/app-users/u1/restrictions/hosted_ai/lift"),
        200,
        json!({"lifted":true}),
    )
    .await;
    client(&server)
        .lift_app_user_restriction("org", "app", "u1", RestrictionScope::HostedAi)
        .await
        .unwrap();
    expect_bearer(&server).await;

    server.reset().await;
    mount(&server, "POST", format!("{APP}/app-users/u1/restrictions/invites/lift"), 404,
        json!({"status":404,"code":"restriction_not_found","error":"restriction_not_found","message":"none"})).await;
    let err = client(&server)
        .lift_app_user_restriction("org", "app", "u1", RestrictionScope::Invites)
        .await
        .unwrap_err();
    assert!(err.is_restriction_not_found());
}

#[tokio::test]
async fn suspend_and_unsuspend() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/app-users/u1/suspend")))
        .and(body_json(json!({"reason":"abuse"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"suspended":true,"since":"t"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let s = client(&server)
        .suspend_app_user("org", "app", "u1", "abuse")
        .await
        .unwrap();
    assert_eq!((s.suspended, s.since.as_deref()), (true, Some("t")));
    expect_bearer(&server).await;

    server.reset().await;
    mount(
        &server,
        "POST",
        format!("{APP}/app-users/u1/suspend"),
        409,
        json!({"status":409,"code":"already_suspended","error":"already_suspended","message":"x"}),
    )
    .await;
    let err = client(&server)
        .suspend_app_user("org", "app", "u1", "")
        .await
        .unwrap_err();
    assert!(err.is_already_suspended());

    server.reset().await;
    mount(
        &server,
        "POST",
        format!("{APP}/app-users/u1/unsuspend"),
        200,
        json!({"suspended":false}),
    )
    .await;
    let s = client(&server)
        .app("org", "app")
        .trust()
        .unsuspend_app_user("u1")
        .await
        .unwrap();
    assert!(!s.suspended && s.since.is_none());

    server.reset().await;
    mount(
        &server,
        "POST",
        format!("{APP}/app-users/u1/unsuspend"),
        409,
        json!({"status":409,"code":"not_suspended","error":"not_suspended","message":"x"}),
    )
    .await;
    let err = client(&server)
        .unsuspend_app_user("org", "app", "u1")
        .await
        .unwrap_err();
    assert!(err.is_not_suspended());
}

#[tokio::test]
async fn invalid_arguments_fail_before_the_network() {
    let server = MockServer::start().await; // no mocks: any request would 404
    let c = client(&server);
    let bad_ref = ModerateRequest::new(ModerationPurpose::Comment, "no-colon", "x");
    let long = ModerateRequest::new(ModerationPurpose::Comment, "a:b", "x".repeat(20_001));
    let unknown = ModerateRequest::new(ModerationPurpose::Unknown, "a:b", "x");
    for r in [&bad_ref, &long, &unknown] {
        assert!(matches!(
            c.moderate("org", "app", r).await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    let mut ok = ModerateRequest::new(ModerationPurpose::Comment, "a:b", "x".repeat(20_000));
    ok.subject_ref = "a_b:C.d-1".into();
    assert!(!matches!(
        c.moderate("org", "app", &ok).await,
        Err(CopepodError::InvalidArgument(_))
    ));

    let mut r = report();
    r.details = "d".repeat(4_001);
    assert!(matches!(
        c.submit_content_report("org", "app", &r).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    let mut r = report();
    r.reporter_ip_hash = "short".into();
    assert!(matches!(
        c.submit_content_report("org", "app", &r).await,
        Err(CopepodError::InvalidArgument(_))
    ));

    let sig = AbuseSignalRequest {
        user_id: String::new(),
        kind: AbuseSignalKind::ChatHarm,
        evidence_ref: String::new(),
    };
    assert!(matches!(
        c.record_abuse_signal("org", "app", &sig).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.get_app_user_trust("org", "app", "").await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.suspend_app_user("org", "app", "u", &"r".repeat(501))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.lift_app_user_restriction("org", "app", "u", RestrictionScope::Account)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.list_trust_actions("org", "app", None, 0).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.list_trust_actions("org", "app", None, 101).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(
        server.received_requests().await.unwrap().len() <= 1,
        "only the valid 20,000-char call may reach the server"
    );
}

#[tokio::test]
async fn unknown_enum_values_deserialise() {
    let r: ModerateResponse = serde_json::from_value(json!({
        "decision":"quarantine","reasons":[],"engine":"gpt9","confidence":0.1,
        "decision_id":"td_2","degraded":false,"restriction":null}))
    .unwrap();
    assert_eq!(r.decision, ModerationDecision::Unknown);
    assert_eq!(r.engine, ModerationEngine::Unknown);
}

#[tokio::test]
async fn key_routes_need_an_api_key() {
    let c = CopepodClient::builder()
        .base_url("http://localhost:1")
        .token("t")
        .auto_refresh(false)
        .build()
        .unwrap();
    assert!(matches!(
        c.moderate("org", "app", &check()).await,
        Err(CopepodError::Auth(_))
    ));
}
