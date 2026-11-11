use copepod_sdk::{
    CopepodClient, CopepodError, CreateOwnerIssue, CreateOwnerSubscriber, OwnerAudienceStatus,
    OwnerIssueStatus, OwnerPage, OwnerSubscriberStatus, PutOwnerAudience, SubscriberQuery,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BASE: &str = "/api/platform/orgs/org_1/apps/app_1/owner-audiences/owner_1";
const ME: &str = "/api/platform/orgs/org_1/apps/app_1/auth/users/me/audience";

fn api_key_client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("platform-token-that-must-not-be-sent")
        .api_key("app-secret")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn bearer_client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn audience_json() -> serde_json::Value {
    json!({
        "owner_user_id": "owner_1",
        "display_name": "Ada's notes",
        "site_url": "https://ada.oikonotes.com",
        "reply_to_owner": false,
        "consent_text": "Email me new posts from Ada's notes. Unsubscribe any time.",
        "enabled": true,
        "status": "active",
        "limits": {"subscribers_max": 1000, "issues_per_month": 4},
        "counts": {"pending": 2, "confirmed": 10, "unsubscribed": 1},
        "issues_this_month": 1,
        "disabled_at": null
    })
}

fn put_request() -> PutOwnerAudience {
    PutOwnerAudience {
        display_name: "Ada's notes".into(),
        site_url: "https://ada.oikonotes.com".into(),
        reply_to_owner: false,
        consent_text: "Email me new posts from Ada's notes. Unsubscribe any time.".into(),
        enabled: true,
    }
}

#[tokio::test]
async fn put_owner_audience_sends_api_key_only() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(BASE))
        .and(header("X-API-Key", "app-secret"))
        .and(body_json(json!({
            "display_name": "Ada's notes",
            "site_url": "https://ada.oikonotes.com",
            "reply_to_owner": false,
            "consent_text": "Email me new posts from Ada's notes. Unsubscribe any time.",
            "enabled": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(audience_json()))
        .expect(1)
        .mount(&server)
        .await;

    let audience = api_key_client(&server)
        .put_owner_audience("org_1", "app_1", "owner_1", &put_request())
        .await
        .unwrap();
    assert_eq!(audience.status, OwnerAudienceStatus::Active);
    assert_eq!(audience.limits.subscribers_max, 1000);
    assert_eq!(audience.counts.confirmed, 10);

    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn create_owner_subscriber_sends_api_key_only_and_decodes_202() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/subscribers")))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"accepted": true})))
        .expect(1)
        .mount(&server)
        .await;

    let accepted = api_key_client(&server)
        .create_owner_subscriber(
            "org_1",
            "app_1",
            "owner_1",
            &CreateOwnerSubscriber {
                email: "reader@example.com".into(),
                consent_text: "consent".into(),
                client_ip: "203.0.113.9".into(),
                source_path: Some("/how-i-take-notes".into()),
            },
        )
        .await
        .unwrap();
    assert!(accepted.accepted);

    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn create_owner_issue_returns_201_draft() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/issues")))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "issue_1",
            "status": "draft",
            "subject": "New: How I take notes",
            "content_hash": "abc123",
            "created_at": "2026-09-25T10:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let issue = api_key_client(&server)
        .create_owner_issue(
            "org_1",
            "app_1",
            "owner_1",
            &CreateOwnerIssue {
                subject: "New: How I take notes".into(),
                body_html: "<p>…</p>".into(),
                body_source: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(issue.status, OwnerIssueStatus::Draft);
}

#[tokio::test]
async fn test_and_send_owner_issue_hit_the_right_routes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/issues/issue_1/test")))
        .and(header("X-API-Key", "app-secret"))
        .and(body_json(json!({"to": "owner@example.com"})))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"accepted": true})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/issues/issue_1/send")))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"accepted": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = api_key_client(&server);
    assert!(
        client
            .test_owner_issue("org_1", "app_1", "owner_1", "issue_1", "owner@example.com")
            .await
            .unwrap()
            .accepted
    );
    assert!(
        client
            .send_owner_issue("org_1", "app_1", "owner_1", "issue_1")
            .await
            .unwrap()
            .accepted
    );
}

#[tokio::test]
async fn me_audience_uses_bearer_only() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ME))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audience_json()))
        .expect(1)
        .mount(&server)
        .await;

    let audience = bearer_client(&server)
        .me_audience("org_1", "app_1", "users")
        .await
        .unwrap();
    assert_eq!(audience.owner_user_id, "owner_1");

    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("x-api-key"));
}

#[tokio::test]
async fn me_audience_subscribers_passes_the_filter_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/subscribers")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(query_param("status", "confirmed"))
        .and(query_param("q", "ada"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{
                "id": "sub_1",
                "email": "reader@example.com",
                "status": "confirmed",
                "created_at": "2026-09-20T09:00:00Z",
                "confirmed_at": "2026-09-20T09:05:00Z",
                "unsubscribed_at": null
            }],
            "total": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page: OwnerPage<copepod_sdk::OwnerSubscriber> = bearer_client(&server)
        .me_audience_subscribers(
            "org_1",
            "app_1",
            "users",
            &SubscriberQuery {
                status: Some(OwnerSubscriberStatus::Confirmed),
                q: Some("ada".into()),
                limit: Some(50),
                offset: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].status, OwnerSubscriberStatus::Confirmed);
}

#[tokio::test]
async fn me_audience_subscribers_csv_returns_text() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/subscribers.csv")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "email,status,created_at,confirmed_at,unsubscribed_at\nreader@example.com,confirmed,2026-09-20T09:00:00Z,2026-09-20T09:05:00Z,\n",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let csv = bearer_client(&server)
        .me_audience_subscribers_csv("org_1", "app_1", "users", &SubscriberQuery::default())
        .await
        .unwrap();
    assert!(csv.starts_with("email,status"));
    assert!(csv.contains("reader@example.com"));
}

#[tokio::test]
async fn me_delete_audience_subscriber_uses_delete_and_204() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("{ME}/subscribers/sub_1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    bearer_client(&server)
        .me_delete_audience_subscriber("org_1", "app_1", "users", "sub_1")
        .await
        .unwrap();
}

#[tokio::test]
async fn me_audience_issues_pages_with_counts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{ME}/issues")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(query_param("limit", "20"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{
                "id": "issue_1",
                "status": "sent",
                "subject": "New: How I take notes",
                "content_hash": "abc123",
                "created_at": "2026-09-25T10:00:00Z",
                "counts": {"sent": 10, "skipped_unsubscribed": 1, "skipped_unconfirmed": 2, "failed": 0}
            }],
            "total": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let page = bearer_client(&server)
        .me_audience_issues("org_1", "app_1", "users", 20, 0)
        .await
        .unwrap();
    assert_eq!(page.items[0].counts.as_ref().unwrap().sent, 10);
}

#[tokio::test]
async fn coded_errors_decode_as_api_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/subscribers")))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "subscriber_limit_reached",
            "message": "subscriber_limit_reached: 1000"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = api_key_client(&server)
        .create_owner_subscriber(
            "org_1",
            "app_1",
            "owner_1",
            &CreateOwnerSubscriber {
                email: "reader@example.com".into(),
                consent_text: "consent".into(),
                client_ip: "203.0.113.9".into(),
                source_path: None,
            },
        )
        .await;
    match result {
        Err(CopepodError::Api {
            status,
            code,
            message,
        }) => {
            assert_eq!(status, 409);
            assert_eq!(code.as_deref(), Some("subscriber_limit_reached"));
            assert!(message.contains("1000"));
        }
        other => panic!("expected CopepodError::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn scoped_owner_audience_wrapper_binds_org_and_app() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ME))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(audience_json()))
        .expect(1)
        .mount(&server)
        .await;

    let client = bearer_client(&server);
    let audience = client
        .app("org_1", "app_1")
        .owner_audience()
        .me("users")
        .await
        .unwrap();
    assert_eq!(audience.display_name, "Ada's notes");
}
