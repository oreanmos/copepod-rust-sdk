use copepod_sdk::{CopepodClient, CopepodError};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// -- Client builder tests --

#[test]
fn test_builder_missing_base_url() {
    let result = CopepodClient::builder().build();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CopepodError::Auth(_)));
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
        .and(path("/api/auth/login"))
        .and(body_json(json!({ "email": "user@test.com", "password": "secret" })))
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
        .base_url(&server.uri())
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
        .and(path("/api/auth/login"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "code": "invalid_credentials",
            "message": "Invalid email or password"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
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
        .and(path("/api/auth/logout"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
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
        .and(path("/api/orgs"))
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
        .base_url(&server.uri())
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
        .and(path("/api/orgs/o1/apps/a1/collections/posts/records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "rec1",
            "title": "Hello",
            "body": "World"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/orgs/o1/apps/a1/collections/posts/records/rec1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "rec1",
            "title": "Hello",
            "body": "World"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    // Create
    let created: serde_json::Value = client
        .create_record("o1", "a1", "posts", &json!({ "title": "Hello", "body": "World" }))
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
        .and(path("/api/orgs/o1/apps/a1/collections/posts/records"))
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
        .base_url(&server.uri())
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
async fn test_delete_record() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/orgs/o1/apps/a1/collections/posts/records/rec1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
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
        .and(path("/api/orgs/none"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "code": "not_found",
            "message": "Organization not found"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
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
        .and(path("/api/orgs"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "message": "Internal server error"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let result = client.list_orgs().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CopepodError::Api { status, message, .. } => {
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
        .and(path("/api/orgs/o1/apps/a1/collections/images/records/r1/files/photo.jpg"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0xFF, 0xD8, 0xFF, 0xE0]) // JPEG magic bytes
        )
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
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
        .and(path("/api/auth/refresh"))
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
        .and(path("/api/orgs"))
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
        .base_url(&server.uri())
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
        .and(path("/api/auth/mfa/verify"))
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
        .base_url(&server.uri())
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
        .and(path("/api/orgs/o1/apps/a1/collections"))
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
        .and(path("/api/orgs/o1/apps/a1/collections/col1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
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
        .and(path("/api/orgs/o1/apps/a1/tickets"))
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
        .and(path("/api/orgs/o1/apps/a1/tickets/t1/comments"))
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
        .base_url(&server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let ticket = client
        .create_ticket("o1", "a1", &json!({
            "subject": "Bug report",
            "description": "Something is broken",
            "priority": "high"
        }))
        .await
        .unwrap();
    assert_eq!(ticket.subject, "Bug report");

    let comment = client
        .add_comment("o1", "a1", "t1", &json!({ "body": "Looking into it" }))
        .await
        .unwrap();
    assert_eq!(comment.body, "Looking into it");
}
