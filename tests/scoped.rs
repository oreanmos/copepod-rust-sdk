use copepod_sdk::{AppLoginResult, CopepodClient};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn scoped_record_client_uses_bound_org_and_app() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o1/apps/a1/records/notes"))
        .and(body_json(json!({ "title": "Scoped note" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "rec_1",
            "title": "Scoped note"
        })))
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(&server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let created = client
        .app("o1", "a1")
        .records("notes")
        .create(&json!({ "title": "Scoped note" }))
        .await
        .unwrap();

    assert_eq!(created["id"], "rec_1");
    assert_eq!(created["title"], "Scoped note");
}

#[tokio::test]
async fn scoped_auth_client_reuses_bound_app_context() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/o1/apps/a1/auth/users/auth-with-password",
        ))
        .and(body_json(json!({
            "identity": "user@example.com",
            "password": "secret"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "access_1",
            "refresh_token": "refresh_1",
            "user": {
                "id": "user_1",
                "email": "user@example.com",
                "name": "User One",
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

    let result = client
        .org("o1")
        .app("a1")
        .auth("users")
        .login("user@example.com", "secret")
        .await
        .unwrap();

    match result {
        AppLoginResult::Success(auth) => {
            assert_eq!(auth.user.email, "user@example.com");
            assert_eq!(auth.token, "access_1");
        }
        AppLoginResult::MfaRequired(_) => panic!("expected successful auth response"),
    }
}
