//! Owner-audience contract details: client-side validation, coded errors from
//! the server, forward-compatible enums and the scoped wrapper for every method.

use copepod_sdk::{
    CopepodClient, CopepodError, CreateOwnerIssue, CreateOwnerSubscriber, OwnerAudience,
    OwnerAudienceLimits, OwnerIssueStatus, OwnerPage, OwnerSubscriber, OwnerSubscriberStatus,
    PutOwnerAudience, SubscriberQuery,
};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BASE: &str = "/api/platform/orgs/org_1/apps/app_1/owner-audiences/owner_1";
const ME: &str = "/api/platform/orgs/org_1/apps/app_1/auth/users/me/audience";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("app-secret")
        .auto_refresh(false)
        .build()
        .unwrap()
}

/// A server that fails the test if any request reaches it.
async fn silent_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    server
}

fn put() -> PutOwnerAudience {
    PutOwnerAudience {
        display_name: "Ada's notes".into(),
        site_url: "https://ada.oikonotes.com".into(),
        reply_to_owner: false,
        consent_text: "Email me new posts.".into(),
        enabled: true,
    }
}

fn signup() -> CreateOwnerSubscriber {
    CreateOwnerSubscriber {
        email: "reader@example.com".into(),
        consent_text: "Email me new posts.".into(),
        client_ip: "203.0.113.9".into(),
        source_path: None,
    }
}

fn issue() -> CreateOwnerIssue {
    CreateOwnerIssue {
        subject: "Hello".into(),
        body_html: "<p>Hi</p>".into(),
        body_source: None,
    }
}

fn code_of<T: std::fmt::Debug>(r: Result<T, CopepodError>) -> (u16, String) {
    match r {
        Err(CopepodError::Api { status, code, .. }) => (status, code.unwrap_or_default()),
        other => panic!("expected Api error, got {other:?}"),
    }
}

fn assert_invalid<T: std::fmt::Debug>(result: Result<T, CopepodError>) {
    assert!(
        matches!(result, Err(CopepodError::InvalidArgument(_))),
        "expected InvalidArgument, got {result:?}"
    );
}

#[tokio::test]
async fn put_rejects_what_the_server_refuses() {
    let server = silent_server().await;
    let c = client(&server);
    let call = |req: PutOwnerAudience| {
        let c = c.clone();
        async move {
            c.put_owner_audience("org_1", "app_1", "owner_1", &req)
                .await
        }
    };
    assert_invalid(
        call(PutOwnerAudience {
            display_name: "  ".into(),
            ..put()
        })
        .await,
    );
    assert_invalid(
        call(PutOwnerAudience {
            display_name: "x".repeat(81),
            ..put()
        })
        .await,
    );
    assert_invalid(
        call(PutOwnerAudience {
            site_url: "http://a.example".into(),
            ..put()
        })
        .await,
    );
    assert_invalid(
        call(PutOwnerAudience {
            site_url: format!("https://{}", "a".repeat(200)),
            ..put()
        })
        .await,
    );
    assert_invalid(
        call(PutOwnerAudience {
            consent_text: "".into(),
            ..put()
        })
        .await,
    );
    assert_invalid(
        call(PutOwnerAudience {
            consent_text: "x".repeat(301),
            ..put()
        })
        .await,
    );
    assert_invalid(
        c.put_owner_audience("org/1", "app_1", "owner_1", &put())
            .await,
    );
}

#[tokio::test]
async fn signup_rejects_what_the_server_refuses() {
    let server = silent_server().await;
    let c = client(&server);
    let call = |req: CreateOwnerSubscriber| {
        let c = c.clone();
        async move {
            c.create_owner_subscriber("org_1", "app_1", "owner_1", &req)
                .await
        }
    };
    assert_invalid(
        call(CreateOwnerSubscriber {
            email: "nope".into(),
            ..signup()
        })
        .await,
    );
    let long = format!("{}@example.com", "a".repeat(250));
    assert_invalid(
        call(CreateOwnerSubscriber {
            email: long,
            ..signup()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerSubscriber {
            consent_text: " ".into(),
            ..signup()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerSubscriber {
            consent_text: "x".repeat(301),
            ..signup()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerSubscriber {
            client_ip: "1.1.1.1, 2.2.2.2".into(),
            ..signup()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerSubscriber {
            client_ip: "".into(),
            ..signup()
        })
        .await,
    );
}

#[tokio::test]
async fn issue_and_test_reject_what_the_server_refuses() {
    let server = silent_server().await;
    let c = client(&server);
    let call = |req: CreateOwnerIssue| {
        let c = c.clone();
        async move {
            c.create_owner_issue("org_1", "app_1", "owner_1", &req)
                .await
        }
    };
    assert_invalid(
        call(CreateOwnerIssue {
            subject: "".into(),
            ..issue()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerIssue {
            subject: "a\nb".into(),
            ..issue()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerIssue {
            subject: "x".repeat(151),
            ..issue()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerIssue {
            body_html: " ".into(),
            ..issue()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerIssue {
            body_html: "a".repeat(200 * 1024 + 1),
            ..issue()
        })
        .await,
    );
    assert_invalid(
        call(CreateOwnerIssue {
            body_html: "a\0b".into(),
            ..issue()
        })
        .await,
    );
    assert_invalid(
        c.test_owner_issue("org_1", "app_1", "owner_1", "issue_1", "not-an-email")
            .await,
    );
    assert_invalid(
        c.test_owner_issue("org_1", "app_1", "owner_1", "../x", "o@example.com")
            .await,
    );
    assert_invalid(c.send_owner_issue("org_1", "app_1", "owner_1", "a/b").await);
}

#[tokio::test]
async fn me_methods_reject_bad_ids_and_unknown_status_filter() {
    let server = silent_server().await;
    let c = client(&server);
    let unknown = SubscriberQuery {
        status: Some(OwnerSubscriberStatus::Unknown),
        ..Default::default()
    };
    assert_invalid(
        c.me_audience_subscribers("org_1", "app_1", "users", &unknown)
            .await,
    );
    assert_invalid(
        c.me_audience_subscribers_csv("org_1", "app_1", "users", &unknown)
            .await,
    );
    assert_invalid(c.me_audience("org_1", "app_1", "us/ers").await);
    assert_invalid(
        c.me_delete_audience_subscriber("org_1", "app_1", "users", "a/b")
            .await,
    );
    assert_invalid(c.me_audience_issues("org_1", "app_1", "u s", 20, 0).await);
}

#[tokio::test]
async fn test_and_send_surface_issue_cancelled_and_upgrading() {
    let server = MockServer::start().await;
    for (suffix, status, code) in [
        ("test", 409, "issue_cancelled"),
        ("send", 409, "issue_cancelled"),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("{BASE}/issues/cancelled/{suffix}")))
            .respond_with(
                ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": code})),
            )
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("PUT"))
        .and(path(BASE))
        .respond_with(ResponseTemplate::new(503).set_body_json(
            json!({"code": "owner_audience_upgrading", "message": "owner_audience_upgrading"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let t: Result<copepod_sdk::OwnerAccepted, _> = c
        .test_owner_issue("org_1", "app_1", "owner_1", "cancelled", "o@example.com")
        .await;
    assert_eq!(code_of(t), (409, "issue_cancelled".into()));
    let s: Result<copepod_sdk::OwnerAccepted, _> = c
        .send_owner_issue("org_1", "app_1", "owner_1", "cancelled")
        .await;
    assert_eq!(code_of(s), (409, "issue_cancelled".into()));
    let p: Result<OwnerAudience, _> = c
        .put_owner_audience("org_1", "app_1", "owner_1", &put())
        .await;
    assert_eq!(code_of(p), (503, "owner_audience_upgrading".into()));
}

#[test]
fn unlimited_limits_and_unknown_statuses_decode() {
    let audience: OwnerAudience = serde_json::from_value(json!({
        "owner_user_id": "owner_1", "display_name": "A", "site_url": "https://a.example",
        "reply_to_owner": false, "consent_text": "c", "enabled": true, "status": "archived",
        "limits": {"subscribers_max": u64::MAX, "issues_per_month": u64::MAX},
        "counts": {"pending": 0, "confirmed": 0, "unsubscribed": 0},
        "issues_this_month": 0
    }))
    .unwrap();
    assert_eq!(
        audience.limits.subscribers_max,
        OwnerAudienceLimits::UNLIMITED
    );
    assert_eq!(audience.status, copepod_sdk::OwnerAudienceStatus::Unknown);
    assert!(audience.disabled_at.is_none());

    let page: OwnerPage<OwnerSubscriber> = serde_json::from_value(json!({
        "items": [{"id": "s1", "email": "r@example.com", "status": "bounced", "created_at": "t"}],
        "total": 1
    }))
    .unwrap();
    assert_eq!(page.items[0].status, OwnerSubscriberStatus::Unknown);

    let cancelled: OwnerIssueStatus = serde_json::from_value(json!("cancelled")).unwrap();
    assert_eq!(cancelled, OwnerIssueStatus::Cancelled);
    assert_eq!(
        serde_json::from_value::<OwnerIssueStatus>(json!("paused")).unwrap(),
        OwnerIssueStatus::Unknown
    );
}

#[test]
fn reply_to_owner_defaults_to_false_when_absent() {
    let req: PutOwnerAudience = serde_json::from_value(json!({
        "display_name": "A", "site_url": "https://a.example", "consent_text": "c", "enabled": true
    }))
    .unwrap();
    assert!(!req.reply_to_owner);
}

#[tokio::test]
async fn scoped_wrapper_reaches_every_method() {
    let server = MockServer::start().await;
    let audience = json!({
        "owner_user_id": "owner_1", "display_name": "A", "site_url": "https://a.example",
        "reply_to_owner": false, "consent_text": "c", "enabled": true, "status": "active",
        "limits": {"subscribers_max": 1, "issues_per_month": 1},
        "counts": {"pending": 0, "confirmed": 0, "unsubscribed": 0},
        "issues_this_month": 0, "disabled_at": null
    });
    let draft = json!({"id": "issue_1", "status": "draft", "subject": "Hello", "content_hash": "h", "created_at": "t"});
    let accepted = json!({"accepted": true});
    let page = json!({"items": [], "total": 0});
    let mocks: Vec<(&str, String, u16, serde_json::Value, &str)> = vec![
        ("PUT", BASE.into(), 200, audience.clone(), "X-API-Key"),
        (
            "POST",
            format!("{BASE}/subscribers"),
            202,
            accepted.clone(),
            "X-API-Key",
        ),
        ("POST", format!("{BASE}/issues"), 201, draft, "X-API-Key"),
        (
            "POST",
            format!("{BASE}/issues/issue_1/test"),
            202,
            accepted.clone(),
            "X-API-Key",
        ),
        (
            "POST",
            format!("{BASE}/issues/issue_1/send"),
            202,
            accepted,
            "X-API-Key",
        ),
        ("GET", ME.into(), 200, audience, "authorization"),
        (
            "GET",
            format!("{ME}/subscribers"),
            200,
            page.clone(),
            "authorization",
        ),
        ("GET", format!("{ME}/issues"), 200, page, "authorization"),
    ];
    for (verb, p, status, body, auth) in mocks {
        let value = if auth == "authorization" {
            "Bearer user-bearer"
        } else {
            "app-secret"
        };
        Mock::given(method(verb))
            .and(path(p))
            .and(header(auth, value))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path(format!("{ME}/subscribers.csv")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_string("email\n"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{ME}/subscribers/s1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let c = client(&server);
    let a = c.app("org_1", "app_1");
    let o = a.owner_audience();
    o.put("owner_1", &put()).await.unwrap();
    assert!(
        o.create_subscriber("owner_1", &signup())
            .await
            .unwrap()
            .accepted
    );
    assert_eq!(
        o.create_issue("owner_1", &issue()).await.unwrap().status,
        OwnerIssueStatus::Draft
    );
    assert!(
        o.test_issue("owner_1", "issue_1", "o@example.com")
            .await
            .unwrap()
            .accepted
    );
    assert!(o.send_issue("owner_1", "issue_1").await.unwrap().accepted);
    o.me("users").await.unwrap();
    assert_eq!(
        o.me_subscribers("users", &SubscriberQuery::default())
            .await
            .unwrap()
            .total,
        0
    );
    assert_eq!(
        o.me_subscribers_csv("users", &SubscriberQuery::default())
            .await
            .unwrap(),
        "email\n"
    );
    o.me_delete_subscriber("users", "s1").await.unwrap();
    assert_eq!(o.me_issues("users", 20, 0).await.unwrap().total, 0);
    // The audience() accessor still points at G1's client, beside owner_audience().
    let _ = a.audience();
}
