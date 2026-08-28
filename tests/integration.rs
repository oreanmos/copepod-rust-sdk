use copepod_sdk::{
    CopepodClient, CopepodError, ImageTransformRequest, RealtimeSubscriptionOptions,
    RecordEventAction, SignedUrlRequest,
};
use futures_util::StreamExt;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// -- Client builder tests --

#[test]
fn test_builder_missing_base_url() {
    let result = CopepodClient::builder().build();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CopepodError::Auth(_)));
}

#[tokio::test]
async fn serving_health_exposes_success_and_stable_election_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/platform/health/serving"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "serving"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .build()
        .unwrap();
    assert_eq!(client.health_serving().await.unwrap().status, "serving");

    server.reset().await;
    Mock::given(method("GET"))
        .and(path("/api/platform/health/serving"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "status": 503,
            "code": "raft_leader_unavailable",
            "message": "Raft leader unavailable; retry shortly"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let error = client.health_serving().await.unwrap_err();
    assert!(error.is_raft_leader_unavailable());
    assert_eq!(
        error.raft_retry_after(),
        Some(std::time::Duration::from_secs(1))
    );
}

#[tokio::test]
async fn app_user_plan_change_preview_uses_auth_context() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/api/platform/orgs/o1/apps/a1/auth/users/me/subscription/change-preview",
        ))
        .and(header("Authorization", "Bearer app-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "current_plan": "basic",
            "target_plan": "pro",
            "direction": "upgrade",
            "current_period_start": "2026-05-01T00:00:00Z",
            "current_period_end": "2026-06-01T00:00:00Z",
            "old_recurring_cents": 400,
            "new_recurring_cents": 900,
            "prorated_charge_cents": 250,
            "currency": "EUR",
            "effective_at": "2026-05-15T00:00:00Z",
            "checkout_required": true
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("app-token")
        .auto_refresh(false)
        .build()
        .unwrap();

    let preview = client
        .preview_app_user_plan_change("o1", "a1", "users", "pro")
        .await
        .unwrap();
    assert_eq!(preview.target_plan, "pro");
    assert_eq!(preview.prorated_charge_cents, 250);
}

#[tokio::test]
async fn app_user_plan_change_submit_posts_consent() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/o1/apps/a1/auth/users/me/subscription/change",
        ))
        .and(header("Authorization", "Bearer app-token"))
        .and(body_json(json!({
            "target_plan": "pro",
            "accepted_terms_version": "2026-05-plan-change-v1",
            "accepted_immediate_service": true,
            "accepted_price": true,
            "redirect_url": "https://example.test/settings/subscription"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "payment_open",
            "preview": {
                "current_plan": "basic",
                "target_plan": "pro",
                "direction": "upgrade",
                "current_period_start": "2026-05-01T00:00:00Z",
                "current_period_end": "2026-06-01T00:00:00Z",
                "old_recurring_cents": 400,
                "new_recurring_cents": 900,
                "prorated_charge_cents": 250,
                "currency": "EUR",
                "effective_at": "2026-05-15T00:00:00Z",
                "checkout_required": true
            },
            "checkout_url": "https://checkout.example/pay",
            "payment_id": "tr_123"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("app-token")
        .auto_refresh(false)
        .build()
        .unwrap();

    let response = client
        .change_app_user_plan(
            "o1",
            "a1",
            "users",
            &copepod_sdk::AppPlanChangeRequest {
                target_plan: "pro".to_string(),
                accepted_terms_version: Some("2026-05-plan-change-v1".to_string()),
                accepted_immediate_service: true,
                accepted_price: true,
                redirect_url: Some("https://example.test/settings/subscription".to_string()),
            },
        )
        .await
        .unwrap();
    assert_eq!(response.status, "payment_open");
    assert_eq!(response.payment_id.as_deref(), Some("tr_123"));
}

#[tokio::test]
async fn app_billing_catalog_parses_public_discounts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/billing/catalog"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "plans": [{
                "id": "plan_basic",
                "name": "Basic",
                "slug": "basic",
                "description": "Basic plan",
                "price_monthly": 400,
                "price_yearly": 3600,
                "currency": "EUR",
                "features": {},
                "included_addons": [],
                "active": true,
                "sort_order": 1
            }],
            "addons": [],
            "discounts": [{
                "id": "discount_beta",
                "name": "Launch beta",
                "code": "BETA",
                "auto_apply": false,
                "show_on_pricing": true,
                "discount_type": "percent",
                "discount_value": 25,
                "duration_cycles": 6,
                "applies_to_plan_slugs": ["basic"],
                "starts_at": null,
                "ends_at": null
            }]
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .build()
        .unwrap();

    let catalog = client.get_app_billing_catalog("o1", "a1").await.unwrap();
    assert_eq!(catalog.plans[0].slug, "basic");
    assert_eq!(catalog.discounts.len(), 1);
    assert_eq!(catalog.discounts[0].code.as_deref(), Some("BETA"));
    assert!(catalog.discounts[0].show_on_pricing);
}

#[test]
fn test_builder_invalid_url() {
    let result = CopepodClient::builder().base_url("not a url").build();
    assert!(result.is_err());
}

#[test]
fn test_builder_with_token() {
    let client = CopepodClient::builder()
        .base_url("http://localhost:8090")
        .token("test-token")
        .refresh_token("test-refresh")
        .build()
        .unwrap();

    // Verify the token store has the token
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pair = rt.block_on(client.token_store().get()).unwrap();
    assert_eq!(pair.token, "test-token");
    assert_eq!(pair.refresh_token, "test-refresh");
}

#[test]
fn test_builder_succeeds_with_base_url() {
    let client = CopepodClient::builder()
        .base_url("http://localhost:8090")
        .build();
    assert!(client.is_ok());
}

// -- Auth flow tests --

#[tokio::test]
async fn test_login_stores_tokens() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/auth/login"))
        .and(body_json(
            json!({ "email": "user@test.com", "password": "secret" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "access-123",
            "refresh_token": "refresh-456",
            "user": {
                "id": "user1",
                "email": "user@test.com",
                "name": "Test User",
                "verified": true,
                "avatar": null,
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .build()
        .unwrap();

    let resp = client.login("user@test.com", "secret").await.unwrap();
    assert_eq!(resp.token, "access-123");
    assert_eq!(resp.refresh_token, "refresh-456");
    assert_eq!(resp.user.email, "user@test.com");

    // Verify stored token
    let pair = client.token_store().get().await.unwrap();
    assert_eq!(pair.token, "access-123");
}

#[tokio::test]
async fn test_login_error_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/auth/login"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "code": "invalid_credentials",
            "message": "Invalid email or password"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .build()
        .unwrap();

    let result = client.login("bad@test.com", "wrong").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CopepodError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 401);
            assert_eq!(code.as_deref(), Some("invalid_credentials"));
            assert_eq!(message, "Invalid email or password");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_logout_clears_token() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/auth/logout"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .refresh_token("ref")
        .auto_refresh(false)
        .build()
        .unwrap();

    assert!(client.token_store().get().await.is_some());
    client.logout().await.unwrap();
    assert!(client.token_store().get().await.is_none());
}

// -- CRUD operations tests --

#[tokio::test]
async fn test_list_orgs() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs"))
        .and(header("Authorization", "Bearer my-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "per_page": 20,
            "total_items": 1,
            "total_pages": 1,
            "items": [{
                "id": "org1",
                "name": "Test Org",
                "slug": "test-org",
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z"
            }]
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("my-token")
        .auto_refresh(false)
        .build()
        .unwrap();

    let result = client.list_orgs().await.unwrap();
    assert_eq!(result.total_items, 1);
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].name, "Test Org");
}

#[tokio::test]
async fn test_create_and_get_record() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/records/posts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "rec1",
            "title": "Hello",
            "body": "World"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/records/posts/rec1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "rec1",
            "title": "Hello",
            "body": "World"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    // Create
    let created: serde_json::Value = client
        .create_record(
            "o1",
            "a1",
            "posts",
            &json!({ "title": "Hello", "body": "World" }),
        )
        .await
        .unwrap();
    assert_eq!(created["id"], "rec1");

    // Get via query builder
    let fetched = client
        .records("o1", "a1", "posts")
        .get_one("rec1")
        .await
        .unwrap();
    assert_eq!(fetched["title"], "Hello");
}

#[tokio::test]
async fn test_list_records_with_query() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/records/posts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "per_page": 10,
            "total_items": 2,
            "total_pages": 1,
            "items": [
                { "id": "r1", "title": "Post 1" },
                { "id": "r2", "title": "Post 2" }
            ]
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let result = client
        .records("o1", "a1", "posts")
        .filter("title != ''")
        .sort("-created")
        .page(1)
        .per_page(10)
        .list()
        .await
        .unwrap();

    assert_eq!(result.total_items, 2);
    assert_eq!(result.items.len(), 2);
}

#[tokio::test]
async fn test_list_launchpads() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/org1/launchpads"))
        .and(header("Authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "lp1",
                "org_id": "org1",
                "name": "Generic App",
                "slug": "generic-app",
                "description": "Reusable deployment flow",
                "status": "draft",
                "version": 1,
                "draft_definition": {
                    "headline": "Deploy",
                    "launch_button_label": "Launch",
                    "create_app": true,
                    "app_defaults": {},
                    "deployment_defaults": {},
                    "source_defaults": {},
                    "domain_defaults": null,
                    "static_env": [],
                    "fields": [],
                    "hook_kind": null
                },
                "published_definition": null,
                "published_at": null,
                "created": "2026-03-09T00:00:00Z",
                "updated": "2026-03-09T00:00:00Z"
            }
        ])))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let launchpads = client.list_launchpads("org1").await.unwrap();
    assert_eq!(launchpads.len(), 1);
    assert_eq!(launchpads[0].slug, "generic-app");
}

#[tokio::test]
async fn test_detect_launchpad_source() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/org1/launchpads/lp1/detect-source"))
        .and(header("Authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "framework": "Axum",
            "port": 3000,
            "health_check_mode": "tcp",
            "memory_request": "128Mi",
            "memory_limit": "256Mi",
            "suggested_env_vars": [
                {
                    "key": "RUST_LOG",
                    "example": "info",
                    "description": "Rust logging level"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let detected = client
        .detect_launchpad_source(
            "org1",
            "lp1",
            &copepod_sdk::LaunchpadLaunchRequest {
                values: std::iter::once((
                    "git_repo_url".to_string(),
                    "https://github.com/example/app".to_string(),
                ))
                .collect(),
            },
        )
        .await
        .unwrap();

    assert_eq!(detected.framework.as_deref(), Some("Axum"));
    assert_eq!(detected.port, Some(3000));
    assert_eq!(detected.suggested_env_vars.len(), 1);
}

#[tokio::test]
async fn test_delete_record() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/platform/orgs/o1/apps/a1/records/posts/rec1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    client
        .delete_record("o1", "a1", "posts", "rec1")
        .await
        .unwrap();
}

// -- Error handling tests --

#[tokio::test]
async fn test_404_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/none"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "code": "not_found",
            "message": "Organization not found"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let result = client.get_org("none").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CopepodError::Api { status, .. } => assert_eq!(status, 404),
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_500_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal server error"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let result = client.list_orgs().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CopepodError::Api {
            status, message, ..
        } => {
            assert_eq!(status, 500);
            assert_eq!(message, "Internal server error");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

// -- File operations tests --

#[tokio::test]
async fn test_download_file() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/api/platform/orgs/o1/apps/a1/files/images/r1/photo.jpg",
        ))
        .and(header("authorization", "Bearer tok"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(vec![0xFF, 0xD8, 0xFF, 0xE0]), // JPEG magic bytes
        )
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let bytes = client
        .download_file("o1", "a1", "images", "r1", "photo.jpg")
        .await
        .unwrap();
    assert_eq!(bytes.len(), 4);
    assert_eq!(bytes[0], 0xFF);
}

#[tokio::test]
async fn test_delete_file() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/api/platform/orgs/o1/apps/a1/files/images/r1/photo.jpg",
        ))
        .and(header("authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    client
        .delete_file("o1", "a1", "images", "r1", "photo.jpg")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_download_file_transformed() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/api/platform/orgs/o1/apps/a1/files/images/r1/photo.jpg",
        ))
        .and(query_param("w", "640"))
        .and(query_param("format", "webp"))
        .and(query_param("q", "82"))
        .and(header("authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1, 2, 3]))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let transform = ImageTransformRequest {
        width: Some(640),
        format: Some("webp".to_string()),
        quality: Some(82),
        ..Default::default()
    };
    let bytes = client
        .download_file_transformed("o1", "a1", "images", "r1", "photo.jpg", &transform)
        .await
        .unwrap();
    assert_eq!(&bytes[..], &[1, 2, 3]);
}

#[tokio::test]
async fn test_create_signed_url_with_transform() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/apps/a1/files/sign"))
        .and(header("authorization", "Bearer tok"))
        .and(body_json(json!({
            "key": "a1/images/r1/photo.jpg",
            "expires_in": 600,
            "transform": {
                "width": 1200,
                "format": "webp",
                "quality": 82
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "url": "/api/platform/apps/a1/files/signed/a1/images/r1/photo.jpg?token=sig&expires=123&w=1200&format=webp&q=82",
            "token": "sig",
            "expires": 123,
            "transform": "w=1200&format=webp&q=82"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let request = SignedUrlRequest {
        key: "a1/images/r1/photo.jpg".to_string(),
        expires_in: Some(600),
        transform: Some(ImageTransformRequest {
            width: Some(1200),
            format: Some("webp".to_string()),
            quality: Some(82),
            ..Default::default()
        }),
    };
    let response = client.create_file_signed_url("a1", &request).await.unwrap();
    assert_eq!(response.token.as_deref(), Some("sig"));
    assert_eq!(response.expires, Some(123));
    assert_eq!(
        response.transform.as_deref(),
        Some("w=1200&format=webp&q=82")
    );
}

// -- Error display tests --

#[test]
fn test_error_display() {
    let err = CopepodError::Api {
        status: 403,
        code: Some("forbidden".into()),
        message: "Access denied".into(),
    };
    assert_eq!(err.to_string(), "API error 403: Access denied");

    let err = CopepodError::Auth("No token".into());
    assert_eq!(err.to_string(), "Auth error: No token");

    let err = CopepodError::Sse("connection lost".into());
    assert_eq!(err.to_string(), "SSE error: connection lost");
}

// -- Auto-refresh test --

#[tokio::test]
async fn test_auto_refresh_on_expiring_token() {
    use chrono::{Duration, Utc};
    use copepod_sdk::auth::TokenPair;

    let server = MockServer::start().await;

    // Mock refresh endpoint
    Mock::given(method("POST"))
        .and(path("/api/platform/auth/refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "new-access-token",
            "refresh_token": "new-refresh-token",
            "user": {
                "id": "u1",
                "email": "test@test.com",
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    // Mock orgs endpoint
    Mock::given(method("GET"))
        .and(path("/api/platform/orgs"))
        .and(header("Authorization", "Bearer new-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "per_page": 20,
            "total_items": 0,
            "total_pages": 0,
            "items": []
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("old-expiring-token")
        .refresh_token("valid-refresh")
        .auto_refresh(true)
        .build()
        .unwrap();

    // Set the token to expire in 30 seconds (within the 60s refresh threshold)
    client
        .token_store()
        .set(TokenPair {
            token: "old-expiring-token".into(),
            refresh_token: "valid-refresh".into(),
            expires_at: Some(Utc::now() + Duration::seconds(30)),
        })
        .await;

    // This should trigger auto-refresh, then use the new token
    let result = client.list_orgs().await.unwrap();
    assert_eq!(result.total_items, 0);

    // Verify token was updated
    let pair = client.token_store().get().await.unwrap();
    assert_eq!(pair.token, "new-access-token");
}

// -- MFA verify test --

#[tokio::test]
async fn test_mfa_verify() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/auth/mfa/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "mfa-access",
            "refresh_token": "mfa-refresh",
            "user": {
                "id": "u1",
                "email": "user@test.com",
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .build()
        .unwrap();

    let resp = client.mfa_verify("mfa-token-123", "123456").await.unwrap();
    assert_eq!(resp.token, "mfa-access");
}

// -- Collections test --

#[tokio::test]
async fn test_crud_collections() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/collections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1,
            "per_page": 20,
            "total_items": 1,
            "total_pages": 1,
            "items": [{
                "id": "col1",
                "name": "posts",
                "collection_type": "base",
                "app_id": "a1",
                "fields": [{ "name": "title", "type": "text", "required": true, "unique": false }],
                "indexes": [],
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z"
            }]
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/platform/orgs/o1/apps/a1/collections/col1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let cols = client.list_collections("o1", "a1").await.unwrap();
    assert_eq!(cols.items.len(), 1);
    assert_eq!(cols.items[0].name, "posts");

    client.delete_collection("o1", "a1", "col1").await.unwrap();
}

// -- Tickets test --

#[tokio::test]
async fn test_ticket_workflow() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/tickets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "t1",
            "ticket_number": "OIKO-20260507-ABC123",
            "app_id": "a1",
            "app_name": "Oikonotes",
            "app_slug": "oikonotes",
            "org_id": "o1",
            "user_id": "u1",
            "user_email": "user@example.com",
            "user_name": "User One",
            "subject": "Bug report",
            "description": "Something is broken",
            "category": "bug",
            "status": "open",
            "priority": "high",
            "context": {},
            "comment_count": 0,
            "created": "2024-01-01T00:00:00Z",
            "updated": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/tickets/t1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "c1",
            "ticket_id": "t1",
            "user_id": "u1",
            "user_name": "User One",
            "content": "Looking into it",
            "is_from_support": false,
            "is_internal": false,
            "created": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let ticket = client
        .create_ticket(
            "o1",
            "a1",
            &json!({
                "subject": "Bug report",
                "description": "Something is broken",
                "priority": "high"
            }),
        )
        .await
        .unwrap();
    assert_eq!(ticket.subject, "Bug report");

    let comment = client
        .add_comment("o1", "a1", "t1", &json!({ "content": "Looking into it" }))
        .await
        .unwrap();
    assert_eq!(comment.content, "Looking into it");
}

#[tokio::test]
async fn test_ticket_workflow_accepts_legacy_ticket_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/tickets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "t1",
            "subject": "Bug report",
            "description": "Something is broken",
            "status": "open",
            "priority": "high",
            "created": "2024-01-01T00:00:00Z",
            "updated": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/tickets/t1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "c1",
            "ticket_id": "t1",
            "user_id": "u1",
            "body": "Looking into it",
            "created": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let ticket = client
        .create_ticket(
            "o1",
            "a1",
            &json!({
                "subject": "Bug report",
                "description": "Something is broken",
                "priority": "high"
            }),
        )
        .await
        .unwrap();
    assert_eq!(ticket.subject, "Bug report");
    assert_eq!(ticket.ticket_number, "");
    assert_eq!(ticket.category, "question");

    let comment = client
        .add_comment("o1", "a1", "t1", &json!({ "content": "Looking into it" }))
        .await
        .unwrap();
    assert_eq!(comment.content, "Looking into it");
}

#[tokio::test]
async fn test_ticket_comments_accept_items_only_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/tickets/t1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{
                "id": "c1",
                "ticket_id": "t1",
                "user_id": "u1",
                "body": "Looking into it",
                "created": "2024-01-01T00:00:00Z"
            }]
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let comments = client.list_comments("o1", "a1", "t1").await.unwrap();
    assert_eq!(comments.items.len(), 1);
    assert_eq!(comments.items[0].content, "Looking into it");
}

// -- Deployments status test --

#[tokio::test]
async fn test_get_deployment_status() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/deployments/d1/status"))
        .and(header("Authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "running": true,
            "ready_replicas": 2,
            "desired_replicas": 2,
            "message": "phase: Ready",
            "db_status": "running"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let status = client.get_deployment_status("o1", "d1").await.unwrap();
    assert!(status.running);
    assert_eq!(status.ready_replicas, 2);
    assert_eq!(status.desired_replicas, 2);
    assert_eq!(status.message, "phase: Ready");
}

#[tokio::test]
async fn test_deploy_queued_returns_queue_metadata() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/deployments/d1/deploy"))
        .and(header("Authorization", "Bearer tok"))
        .and(body_json(json!({ "mode": "force" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queued": true,
            "app_id": "d1",
            "log_id": "l1",
            "action": "deploy",
            "status": "pending"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let queued = client.deploy_queued("o1", "d1").await.unwrap();
    assert!(queued.queued);
    assert_eq!(queued.log_id, "l1");
    assert_eq!(queued.action, "deploy");
}

#[tokio::test]
async fn test_deploy_if_image_changed_uses_update_check_mode() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/deployments/d1/deploy"))
        .and(header("Authorization", "Bearer tok"))
        .and(body_json(json!({ "mode": "if_image_changed" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "queued": true,
            "app_id": "d1",
            "log_id": "l2",
            "action": "redeploy",
            "status": "pending"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let queued = client
        .deploy_if_image_changed_queued("o1", "d1")
        .await
        .unwrap();
    assert_eq!(queued.log_id, "l2");
    assert_eq!(queued.action, "redeploy");
}

#[tokio::test]
async fn test_list_deployment_builds_parses_items_wrapper() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/deployments/d1/builds"))
        .and(header("Authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "id": "b1",
                    "deployed_app_id": "d1",
                    "git_source_id": "g1",
                    "commit_sha": "abc123",
                    "commit_message": "feat: update",
                    "branch": "main",
                    "status": "success",
                    "build_method": "dockerfile",
                    "image_tag": "abc123",
                    "duration_ms": 1234,
                    "error_message": null,
                    "created": "2024-01-01T00:00:00Z",
                    "updated": "2024-01-01T00:00:00Z"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let builds = client.list_deployment_builds("o1", "d1").await.unwrap();
    assert_eq!(builds.len(), 1);
    assert_eq!(builds[0].id, "b1");
    assert_eq!(builds[0].build_method, "dockerfile");
}

#[tokio::test]
async fn instance_administrator_lifecycle_uses_explicit_contracts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/platform/instance-administrators"))
        .and(header("Authorization", "Bearer owner-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{
                "user_id": "u1",
                "email": "owner@example.test",
                "name": "Owner",
                "role": "owner",
                "created": "2026-07-22T00:00:00Z",
                "updated": "2026-07-22T00:00:00Z"
            }]
        })))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/api/platform/instance-administrators/u2"))
        .and(body_json(json!({
            "role": "admin",
            "reason": "approved in incident 1234"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "user_id": "u2",
            "role": "admin"
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/platform/instance-administrators/u2"))
        .and(body_json(json!({"reason": "access review completed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "user_id": "u2",
            "revoked": true
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("owner-token")
        .auto_refresh(false)
        .build()
        .unwrap();
    let admins = client.list_instance_administrators().await.unwrap();
    assert_eq!(admins.items[0].role, "owner");
    let granted = client
        .set_instance_role(
            "u2",
            &copepod_sdk::SetInstanceRoleRequest {
                role: "admin".to_string(),
                reason: "approved in incident 1234".to_string(),
            },
        )
        .await
        .unwrap();
    assert_eq!(granted.role, "admin");
    let revoked = client
        .revoke_instance_role(
            "u2",
            &copepod_sdk::RevokeInstanceRoleRequest {
                reason: "access review completed".to_string(),
            },
        )
        .await
        .unwrap();
    assert!(revoked.revoked);
}

#[tokio::test]
async fn owner_recovery_uses_the_break_glass_header_without_session_auth() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/platform/auth/admin-recovery"))
        .and(header(
            "x-copepod-admin-recovery-token",
            "01234567890123456789012345678901",
        ))
        .and(body_json(json!({
            "user_id": "u2",
            "reason": "lost owner credentials incident"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "user_id": "u2",
            "role": "owner",
            "recovered": true
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .build()
        .unwrap();
    let result = client
        .recover_instance_owner(
            "01234567890123456789012345678901",
            &copepod_sdk::RecoverInstanceOwnerRequest {
                user_id: "u2".to_string(),
                reason: "lost owner credentials incident".to_string(),
            },
        )
        .await
        .unwrap();
    assert!(result.recovered);
}

#[tokio::test]
async fn managed_database_backup_and_restore_contracts_are_typed() {
    let server = MockServer::start().await;
    let backup = json!({
        "id": "b1",
        "volume_id": "v1",
        "deployed_app_id": "db1",
        "filename": "postgres-db1-b1.dump",
        "size_bytes": 123,
        "status": "in_progress",
        "destination": "s3-main",
        "destination_meta": {},
        "checksum_sha256": null,
        "error_message": null,
        "created": "2026-07-22T00:00:00Z",
        "updated": "2026-07-22T00:00:00Z",
        "completed_at": null
    });

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/databases/db1/backups"))
        .and(body_json(json!({"destination_id": "s3-main"})))
        .respond_with(ResponseTemplate::new(202).set_body_json(backup.clone()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/o1/databases/db1/backups/b1/restore",
        ))
        .and(body_json(json!({
            "confirmation": "restore:b1",
            "safety_backup_id": "b2"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "id": "r1",
            "backup_id": "b1",
            "safety_backup_id": "b2",
            "deployed_app_id": "db1",
            "status": "pending",
            "error_message": null,
            "created": "2026-07-22T00:00:00Z",
            "updated": "2026-07-22T00:00:00Z",
            "completed_at": null
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/platform/orgs/o1/databases/db1/backups/b1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted": true})))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("operator-token")
        .auto_refresh(false)
        .build()
        .unwrap();
    let queued = client
        .trigger_managed_database_backup(
            "o1",
            "db1",
            &copepod_sdk::TriggerDatabaseBackupRequest {
                destination_id: Some("s3-main".to_string()),
            },
        )
        .await
        .unwrap();
    assert_eq!(queued.status, copepod_sdk::VolumeBackupStatus::InProgress);
    let restored = client
        .restore_managed_database_backup(
            "o1",
            "db1",
            "b1",
            &copepod_sdk::RestoreDatabaseBackupRequest::confirmed("b1", "b2"),
        )
        .await
        .unwrap();
    assert_eq!(restored.backup_id, "b1");
    assert!(
        client
            .delete_managed_database_backup("o1", "db1", "b1")
            .await
            .unwrap()
            .deleted
    );
}

#[tokio::test]
async fn realtime_subscription_uses_header_auth_and_typed_filters() {
    let server = MockServer::start().await;
    let body = concat!(
        "event: ready\n",
        "data: {\"status\":\"connected\"}\n\n",
        "id: 42\n",
        "event: record\n",
        "data: {\"id\":42,\"action\":\"update\",\"collection\":\"notes\",\"record\":{\"id\":\"n1\"}}\n\n",
    );

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/realtime"))
        .and(header("Authorization", "Bearer app-token"))
        .and(header("Last-Event-ID", "41"))
        .and(query_param("collections", "notes"))
        .and(query_param("actions", "update,delete"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .set_body_raw(body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("app-token")
        .auto_refresh(false)
        .build()
        .unwrap();
    let mut stream = client
        .subscribe_with_options(
            "o1",
            "a1",
            RealtimeSubscriptionOptions {
                collections: vec!["notes".to_string()],
                actions: vec![RecordEventAction::Update, RecordEventAction::Delete],
                last_event_id: Some(41),
            },
        )
        .await
        .unwrap();

    let event = stream.next().await.unwrap().unwrap();
    assert_eq!(event.id, 42);
    assert_eq!(event.action, "update");
    assert_eq!(event.collection, "notes");
    assert_eq!(event.record["id"], "n1");

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0]
        .url
        .query_pairs()
        .all(|(name, _)| name != "access_token"));
}

#[tokio::test]
async fn api_key_contract_returns_raw_lists_and_the_one_time_secret() {
    let server = MockServer::start().await;
    let key = json!({
        "id": "key-1",
        "app_id": "a1",
        "name": "reader",
        "key_prefix": "cpd_01234567",
        "scopes": ["records:read"],
        "created": "2026-07-22T00:00:00Z",
        "last_used": null,
        "revoked_at": null
    });

    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/o1/apps/a1/api-keys"))
        .and(header("Authorization", "Bearer admin-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([key.clone()])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/api-keys"))
        .and(body_json(json!({
            "name": "reader",
            "scopes": ["records:read"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut created = key;
            created["key"] = json!("cpd_0123456789abcdef0123456789abcdef0123456789abcdef");
            created
        }))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("admin-token")
        .auto_refresh(false)
        .build()
        .unwrap();
    let keys = client.list_api_keys("o1", "a1").await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].app_id, "a1");

    let created = client
        .create_api_key(
            "o1",
            "a1",
            &copepod_sdk::ApiKeyCreate::new("reader")
                .with_scope(copepod_sdk::API_KEY_SCOPE_RECORDS_READ),
        )
        .await
        .unwrap();
    assert!(created.key.starts_with("cpd_"));
    assert_eq!(created.api_key.key_prefix, "cpd_01234567");
}

#[tokio::test]
async fn shard_move_sends_and_returns_the_assignment_epoch() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/platform/cluster/shards/shard-1/move"))
        .and(header("Authorization", "Bearer platform-admin"))
        .and(body_json(json!({
            "target_group_id": "group-2",
            "expected_assignment_epoch": 11
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "move_id": "move-1",
            "shard_id": "shard-1",
            "target_group_id": "group-2",
            "assignment_epoch": 12,
            "status": "completed"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("platform-admin")
        .auto_refresh(false)
        .build()
        .unwrap();
    let moved = client
        .move_shard(
            "shard-1",
            &copepod_sdk::MoveShardRequest {
                target_group_id: "group-2".to_string(),
                expected_assignment_epoch: 11,
            },
        )
        .await
        .unwrap();
    assert_eq!(moved.assignment_epoch, 12);
}

#[tokio::test]
async fn raft_membership_repair_contracts_are_typed_and_guarded() {
    let server = MockServer::start().await;
    let initial = json!({
        "membership_log_index": 41,
        "leader_node_id": 1,
        "leader_last_applied_index": 900,
        "joint": false,
        "voters": [
            {
                "node_id": 1,
                "address": "copepod-0.copepod-headless:8090",
                "role": "leader",
                "matched_index": 900,
                "caught_up": true
            },
            {
                "node_id": 2,
                "address": "copepod-1.copepod-headless:8090",
                "role": "voter",
                "matched_index": 900,
                "caught_up": true
            },
            {
                "node_id": 3,
                "address": "copepod-2.copepod-headless:8090",
                "role": "voter",
                "matched_index": 700,
                "caught_up": false
            }
        ],
        "learners": []
    });
    let learner_added = json!({
        "membership_log_index": 42,
        "leader_node_id": 1,
        "leader_last_applied_index": 901,
        "joint": false,
        "voters": initial["voters"].clone(),
        "learners": [{
            "node_id": 4,
            "address": "copepod-3.copepod-headless:8090",
            "role": "learner",
            "matched_index": 850,
            "caught_up": false
        }]
    });
    let learner_promoted = json!({
        "membership_log_index": 43,
        "leader_node_id": 1,
        "leader_last_applied_index": 902,
        "joint": false,
        "voters": [
            initial["voters"][0].clone(),
            initial["voters"][1].clone(),
            initial["voters"][2].clone(),
            {
                "node_id": 4,
                "address": "copepod-3.copepod-headless:8090",
                "role": "voter",
                "matched_index": 902,
                "caught_up": true
            }
        ],
        "learners": []
    });
    let stale_member_removed = json!({
        "membership_log_index": 44,
        "leader_node_id": 1,
        "leader_last_applied_index": 903,
        "joint": false,
        "voters": [
            initial["voters"][0].clone(),
            initial["voters"][1].clone(),
            learner_promoted["voters"][3].clone()
        ],
        "learners": []
    });

    Mock::given(method("GET"))
        .and(path("/api/platform/cluster/raft/membership"))
        .and(header("Authorization", "Bearer platform-admin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(initial))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/platform/cluster/raft/learners"))
        .and(header("Authorization", "Bearer platform-admin"))
        .and(body_json(json!({
            "node_id": 4,
            "address": "copepod-3.copepod-headless:8090",
            "expected_membership_log_index": 41
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(learner_added))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/platform/cluster/raft/learners/4/promote"))
        .and(header("Authorization", "Bearer platform-admin"))
        .and(body_json(json!({
            "expected_membership_log_index": 42
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(learner_promoted))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/platform/cluster/raft/members/3/remove"))
        .and(header("Authorization", "Bearer platform-admin"))
        .and(body_json(json!({
            "expected_membership_log_index": 43,
            "expected_voters": [1, 2, 3, 4],
            "allow_temporary_two_voters": false,
            "confirmation": "REMOVE RAFT NODE 3"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(stale_member_removed))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("platform-admin")
        .auto_refresh(false)
        .build()
        .unwrap();

    let membership = client.get_raft_membership().await.unwrap();
    assert_eq!(membership.membership_log_index, Some(41));
    assert_eq!(membership.voter_ids(), vec![1, 2, 3]);
    assert_eq!(
        membership.voters[0].role,
        copepod_sdk::RaftMemberRole::Leader
    );

    let membership = client
        .add_raft_learner(&copepod_sdk::AddRaftLearnerRequest {
            node_id: 4,
            address: "copepod-3.copepod-headless:8090".to_string(),
            expected_membership_log_index: 41,
        })
        .await
        .unwrap();
    assert_eq!(membership.learners[0].node_id, 4);
    assert!(!membership.learners[0].caught_up);

    let membership = client
        .promote_raft_learner(
            4,
            &copepod_sdk::PromoteRaftLearnerRequest {
                expected_membership_log_index: 42,
            },
        )
        .await
        .unwrap();
    assert_eq!(membership.voter_ids(), vec![1, 2, 3, 4]);

    let membership = client
        .remove_raft_member(
            3,
            &copepod_sdk::RemoveRaftMemberRequest::confirmed(3, 43, vec![1, 2, 3, 4]),
        )
        .await
        .unwrap();
    assert_eq!(membership.voter_ids(), vec![1, 2, 4]);
}

#[tokio::test]
async fn root_key_rotation_never_sends_key_material() {
    let server = MockServer::start().await;
    let state = json!({
        "from_fingerprint": "old-fingerprint",
        "to_fingerprint": "new-fingerprint",
        "status": "completed",
        "rotated_values": 14,
        "verified_values": 14,
        "started_by": "owner-1",
        "started": "2026-07-22T00:00:00Z",
        "updated": "2026-07-22T00:01:00Z",
        "completed": "2026-07-22T00:01:00Z"
    });
    Mock::given(method("GET"))
        .and(path("/api/platform/settings/rotate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "primary_fingerprint": "new-fingerprint",
            "previous_configured": true,
            "safe_to_remove_previous": false,
            "rotation": state.clone()
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/platform/settings/rotate"))
        .and(body_json(json!({
            "action": "rotate",
            "confirm_fingerprint": "new-fingerprint"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "rotation": state,
            "primary_fingerprint": "new-fingerprint",
            "safe_to_remove_previous": false
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("owner-token")
        .auto_refresh(false)
        .build()
        .unwrap();
    let status = client.get_secret_rotation_status().await.unwrap();
    assert!(status.previous_configured);
    let result = client
        .rotate_secrets(&copepod_sdk::RotateSecretsRequest {
            action: "rotate".to_string(),
            confirm_fingerprint: "new-fingerprint".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(result.rotation.rotated_values, 14);
}

#[tokio::test]
async fn outbound_webhook_create_is_the_only_response_with_a_secret() {
    let server = MockServer::start().await;
    let webhook = json!({
        "id": "wh-1",
        "app_id": "a1",
        "url": "https://hooks.example.test/copepod",
        "events": ["records.created"],
        "active": true,
        "description": "record sink",
        "created": "2026-07-22T00:00:00Z",
        "updated": "2026-07-22T00:00:00Z"
    });
    Mock::given(method("GET"))
        .and(path("/api/platform/apps/a1/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [webhook.clone()]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/platform/apps/a1/webhooks"))
        .and(body_json(json!({
            "url": "https://hooks.example.test/copepod",
            "events": ["records.created"],
            "description": "record sink"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut created = webhook;
            created["secret"] = json!("a".repeat(64));
            created
        }))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("admin-token")
        .auto_refresh(false)
        .build()
        .unwrap();
    let listed = client.list_webhooks("a1").await.unwrap();
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].description, "record sink");

    let created = client
        .create_webhook(
            "a1",
            &copepod_sdk::OutboundWebhookCreate {
                url: "https://hooks.example.test/copepod".to_string(),
                events: vec!["records.created".to_string()],
                description: "record sink".to_string(),
            },
        )
        .await
        .unwrap();
    assert_eq!(created.secret.len(), 64);
    assert_eq!(created.webhook.id, "wh-1");
}
