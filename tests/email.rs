use copepod_sdk::{
    CopepodClient, CopepodError, EmailDeliveryState, TransactionalEmailRequest,
    MAX_EMAIL_DELIVERY_STATUS_BATCH,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn delivery_json(status: &str) -> serde_json::Value {
    json!({
        "delivery_id": "delivery_1",
        "status": status,
        "attempts": 1,
        "created_at": "2026-07-22T12:00:00Z",
        "updated_at": "2026-07-22T12:01:00Z",
        "sent_at": null,
        "last_error": null
    })
}

fn email_request() -> TransactionalEmailRequest {
    TransactionalEmailRequest {
        to: "traveler@example.com".to_string(),
        subject: "You are invited".to_string(),
        body_html: "<p>Join the trip.</p>".to_string(),
    }
}

#[tokio::test]
async fn enqueue_uses_api_key_and_idempotency_headers_without_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/org_1/apps/app_1/email/send"))
        .and(header("X-API-Key", "app-secret"))
        .and(header("Idempotency-Key", "trip-invite:participant_1:v1"))
        .and(body_json(json!({
            "to": "traveler@example.com",
            "subject": "You are invited",
            "body_html": "<p>Join the trip.</p>"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "delivery_id": "delivery_1",
            "status": "pending",
            "deduplicated": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("platform-token-that-must-not-be-sent")
        .api_key("app-secret")
        .auto_refresh(false)
        .build()
        .unwrap();

    let response = client
        .enqueue_transactional_email(
            "org_1",
            "app_1",
            "trip-invite:participant_1:v1",
            &email_request(),
        )
        .await
        .unwrap();

    assert_eq!(response.delivery_id, "delivery_1");
    assert_eq!(response.status, EmailDeliveryState::Pending);
    assert!(!response.deduplicated);

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn app_scoped_client_covers_status_retry_and_cancel_routes() {
    let server = MockServer::start().await;
    let route = "/api/platform/orgs/org_1/apps/app_1/email/deliveries/delivery_1";

    Mock::given(method("GET"))
        .and(path(route))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(delivery_json("sending")))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path(format!("{route}/retry")))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(delivery_json("pending")))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path(format!("{route}/cancel")))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(delivery_json("cancelled")))
        .expect(1)
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap();
    let email = client.app("org_1", "app_1").email();

    assert_eq!(
        email.get("delivery_1").await.unwrap().status,
        EmailDeliveryState::Sending
    );
    assert_eq!(
        email.retry("delivery_1").await.unwrap().status,
        EmailDeliveryState::Pending
    );
    assert_eq!(
        email.cancel("delivery_1").await.unwrap().status,
        EmailDeliveryState::Cancelled
    );
}

#[tokio::test]
async fn bounded_status_request_uses_contract_body_and_preserves_response_order() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org_1/apps/app_1/email/deliveries/status",
        ))
        .and(header("X-API-Key", "app-secret"))
        .and(body_json(json!({
            "delivery_ids": ["delivery_2", "delivery_1"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "deliveries": [
                {
                    "delivery_id": "delivery_2",
                    "status": "failed",
                    "attempts": 5,
                    "created_at": "2026-07-22T12:00:00Z",
                    "updated_at": "2026-07-22T12:10:00Z",
                    "sent_at": null,
                    "last_error": "provider rejected the request"
                },
                {
                    "delivery_id": "delivery_1",
                    "status": "sent",
                    "attempts": 1,
                    "created_at": "2026-07-22T12:00:00Z",
                    "updated_at": "2026-07-22T12:01:00Z",
                    "sent_at": "2026-07-22T12:01:00Z",
                    "last_error": null
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap();
    let ids = vec!["delivery_2".to_string(), "delivery_1".to_string()];

    let response = client
        .get_email_delivery_statuses("org_1", "app_1", &ids)
        .await
        .unwrap();

    assert_eq!(response.deliveries[0].delivery_id, "delivery_2");
    assert_eq!(response.deliveries[1].delivery_id, "delivery_1");
    assert_eq!(
        response.deliveries[0].last_error.as_deref(),
        Some("provider rejected the request")
    );
    assert!(response.deliveries[1].sent_at.is_some());
}

#[tokio::test]
async fn email_operations_require_an_api_key() {
    let client = CopepodClient::builder()
        .base_url("http://localhost:8090")
        .token("bearer-only")
        .auto_refresh(false)
        .build()
        .unwrap();

    let error = client
        .get_email_delivery("org_1", "app_1", "delivery_1")
        .await
        .unwrap_err();

    assert!(matches!(error, CopepodError::Auth(_)));
}

#[tokio::test]
async fn batch_status_rejects_more_than_one_hundred_ids_without_a_request() {
    let server = MockServer::start().await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap();
    let ids = (0..=MAX_EMAIL_DELIVERY_STATUS_BATCH)
        .map(|index| format!("delivery_{index}"))
        .collect::<Vec<_>>();

    let error = client
        .get_email_delivery_statuses("org_1", "app_1", &ids)
        .await
        .unwrap_err();

    assert!(matches!(error, CopepodError::InvalidArgument(_)));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn invalid_payloads_and_path_ids_are_rejected_without_a_request() {
    let server = MockServer::start().await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap();

    let empty = Vec::new();
    assert!(matches!(
        client
            .get_email_delivery_statuses("org_1", "app_1", &empty)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        client
            .cancel_email_delivery("org_1", "app_1", "../delivery_1")
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));

    let mut invalid_email = email_request();
    invalid_email.subject = "Invite\r\nBcc: victim@example.com".into();
    assert!(matches!(
        client
            .enqueue_transactional_email("org_1", "app_1", "key", &invalid_email)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[test]
fn api_key_is_validated_and_redacted_from_client_debug() {
    let secret = "cpd_a_secret_that_must_not_appear_in_debug_output";
    let client = CopepodClient::builder()
        .base_url("http://localhost:8090")
        .api_key(secret)
        .build()
        .unwrap();
    let debug = format!("{client:?}");

    assert!(!debug.contains(secret), "API key leaked through Debug");
    assert!(debug.contains("<redacted>"));
    assert!(matches!(
        CopepodClient::builder()
            .base_url("http://localhost:8090")
            .api_key(" surrounded ")
            .build(),
        Err(CopepodError::InvalidArgument(_))
    ));
}

#[tokio::test]
async fn retry_conflict_keeps_server_error_details() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org_1/apps/app_1/email/deliveries/delivery_1/retry",
        ))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "email_delivery_not_retryable",
            "message": "only failed deliveries can be retried"
        })))
        .mount(&server)
        .await;
    let client = CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap();

    let error = client
        .retry_email_delivery("org_1", "app_1", "delivery_1")
        .await
        .unwrap_err();

    match error {
        CopepodError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 409);
            assert_eq!(code.as_deref(), Some("email_delivery_not_retryable"));
            assert_eq!(message, "only failed deliveries can be retried");
        }
        other => panic!("expected API conflict, got {other:?}"),
    }
}
