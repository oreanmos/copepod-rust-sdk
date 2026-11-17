//! Contract tests for the analytics ingestion SDK (launch-g2 S8, acceptance
//! criterion #15): API-key auth without a bearer, the exact wire body, offline
//! validation of batch size, event names and props, server error-code
//! preservation, scoped delegation and the serialisation contracts.

use chrono::{TimeZone, Utc};
use copepod_sdk::{
    event_names, AnalyticsEvent, CopepodClient, CopepodError, LeadAcquisition,
    RegisterWithBillingIntentRequest,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const EVENTS: &str = "/api/platform/orgs/org_1/apps/app_1/analytics/events";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap()
}

fn occurred_at() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 20, 9, 12, 3).unwrap()
}

fn event(name: &str) -> AnalyticsEvent {
    AnalyticsEvent {
        name: name.to_string(),
        occurred_at: occurred_at(),
        user_id: Some("user_1".into()),
        path: Some("/pricing".into()),
        referrer: Some("https://news.ycombinator.com/item?id=1".into()),
        ip: Some("203.0.113.9".into()),
        user_agent: Some("Mozilla/5.0".into()),
        acquisition: Some(LeadAcquisition {
            utm_source: Some("hn".into()),
            ..Default::default()
        }),
        props: Some(json!({"plan": "pro"}).as_object().unwrap().clone()),
    }
}

#[tokio::test]
async fn track_events_uses_api_key_and_exact_body_without_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(EVENTS))
        .and(header("X-API-Key", "app-secret"))
        .and(body_json(json!({
            "events": [{
                "name": "page_view",
                "occurred_at": "2026-11-20T09:12:03Z",
                "user_id": "user_1",
                "path": "/pricing",
                "referrer": "https://news.ycombinator.com/item?id=1",
                "ip": "203.0.113.9",
                "user_agent": "Mozilla/5.0",
                "acquisition": {"utm_source": "hn"},
                "props": {"plan": "pro"}
            }]
        })))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(json!({"accepted": 1, "filtered": 0, "rejected": []})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let events = [event(event_names::PAGE_VIEW)];
    let response = client(&server)
        .track_events("org_1", "app_1", &events)
        .await
        .unwrap();
    assert_eq!(response.accepted, 1);
    assert_eq!(response.filtered, 0);
    assert!(response.rejected.is_empty());

    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn track_events_returns_partial_rejections() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(EVENTS))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "accepted": 1,
            "filtered": 1,
            "rejected": [{"index": 1, "code": "invalid_user_id"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let events = [event(event_names::PAGE_VIEW), event(event_names::ACTIVATED)];
    let response = client(&server)
        .track_events("org_1", "app_1", &events)
        .await
        .unwrap();
    assert_eq!(response.accepted, 1);
    assert_eq!(response.filtered, 1);
    assert_eq!(response.rejected.len(), 1);
    assert_eq!(response.rejected[0].index, 1);
    assert_eq!(response.rejected[0].code, "invalid_user_id");
}

#[tokio::test]
async fn track_events_validates_before_network() {
    let server = MockServer::start().await;
    let c = client(&server);

    let mut eleven_props = serde_json::Map::new();
    for i in 0..11 {
        eleven_props.insert(format!("prop_{i}"), json!(i as u64));
    }
    let nested_prop = json!({"nested": {"a": 1}}).as_object().unwrap().clone();

    let cases: [(&str, Vec<AnalyticsEvent>); 5] = [
        ("0 events", vec![]),
        (
            "101 events",
            (0..101).map(|_| event(event_names::PAGE_VIEW)).collect(),
        ),
        // A name only Copepod emits is refused locally, before any request.
        (
            "reserved Copepod-only name",
            vec![event("signup_completed")],
        ),
        (
            "11 props",
            vec![AnalyticsEvent {
                props: Some(eleven_props),
                ..event(event_names::PAGE_VIEW)
            }],
        ),
        (
            "nested prop value",
            vec![AnalyticsEvent {
                props: Some(nested_prop),
                ..event(event_names::PAGE_VIEW)
            }],
        ),
    ];
    for (label, events) in cases {
        match c.track_events("org_1", "app_1", &events).await {
            Err(CopepodError::InvalidArgument(message)) => {
                assert!(!message.is_empty(), "{label}: empty message");
                if label == "reserved Copepod-only name" {
                    assert!(
                        message.contains("Copepod"),
                        "reserved-name message must name the Copepod-emitted source: {message}"
                    );
                }
            }
            other => panic!("{label}: expected InvalidArgument before any request, got {other:?}"),
        }
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn track_events_requires_an_api_key() {
    let server = MockServer::start().await;
    let c = CopepodClient::builder()
        .base_url(server.uri())
        .build()
        .unwrap();
    let events = [event(event_names::PAGE_VIEW)];
    match c.track_events("org_1", "app_1", &events).await {
        Err(CopepodError::Auth(message)) => assert!(!message.is_empty()),
        other => panic!("expected Auth before any request, got {other:?}"),
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn track_events_preserves_server_error_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(EVENTS))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "status": 400,
            "code": "batch_too_large",
            "error": "batch_too_large",
            "message": "batch_too_large: at most 100 events"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let events = [event(event_names::PAGE_VIEW)];
    match client(&server)
        .track_events("org_1", "app_1", &events)
        .await
    {
        Err(CopepodError::Api {
            status,
            code,
            message,
        }) => {
            assert_eq!(status, 400);
            assert_eq!(code.as_deref(), Some("batch_too_large"));
            assert!(!message.is_empty());
        }
        other => panic!("expected Api, got {other:?}"),
    }
}

#[tokio::test]
async fn scoped_analytics_client_delegates() {
    let server = MockServer::start().await;
    let events = [event(event_names::PAGE_VIEW)];
    let body = json!({"events": [serde_json::to_value(&events[0]).unwrap()]});
    Mock::given(method("POST"))
        .and(path(EVENTS))
        .and(header("X-API-Key", "app-secret"))
        .and(body_json(body))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(json!({"accepted": 1, "filtered": 0, "rejected": []})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server)
        .app("org_1", "app_1")
        .analytics()
        .track(&events)
        .await
        .unwrap();
    assert_eq!(response.accepted, 1);

    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[test]
fn occurred_at_serialises_as_rfc3339_utc() {
    let value = serde_json::to_value(event(event_names::PAGE_VIEW)).unwrap();
    assert_eq!(value["occurred_at"], json!("2026-11-20T09:12:03Z"));
}

#[test]
fn register_with_billing_intent_serialises_acquisition_only_when_set() {
    let with = RegisterWithBillingIntentRequest {
        billing_intent_id: "bi_1".into(),
        email: "user@example.com".into(),
        password: "secret".into(),
        acquisition: Some(LeadAcquisition {
            utm_source: Some("hn".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let value = serde_json::to_value(&with).unwrap();
    assert_eq!(value["acquisition"], json!({"utm_source": "hn"}));

    let without = RegisterWithBillingIntentRequest {
        acquisition: None,
        ..with
    };
    let value = serde_json::to_value(&without).unwrap();
    assert!(value.get("acquisition").is_none());
}
