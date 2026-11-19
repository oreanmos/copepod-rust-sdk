//! toolkit-ui4-ui5 S9 review round: id validation, redirects, URL-free
//! errors, redacted tokens, comment status filter.

use copepod_sdk::auth::TokenPair;
use copepod_sdk::{CommentDecision, CopepodClient, CopepodError, SpaceCommentStatus};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("app-key")
        .auto_refresh(false)
        .build()
        .unwrap()
}

#[tokio::test]
async fn backslash_and_non_ascii_ids_never_send_a_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    for bad in [
        "..\\..\\x",
        "a\\b",
        "\\",
        "caf\u{e9}",
        "a\u{202e}b",
        "%2e",
        ".",
    ] {
        let err = c
            .create_shared_comment("org", "app", "tok1", "notes", bad, None, "x", None)
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{bad:?}");
        let err = c
            .decide_space_comment("org", "app", "sp1", bad, CommentDecision::Approve)
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{bad:?}");
        let err = c
            .update_my_profile("org", "app", bad, Some("x"))
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{bad:?}");
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn colon_dash_underscore_dot_tilde_ids_are_allowed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/platform/orgs/org/apps/app/spaces/sp1/comments/note:a-b_c.d~e/decision",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"id": "x", "status": "approved"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .decide_space_comment(
            "org",
            "app",
            "sp1",
            "note:a-b_c.d~e",
            CommentDecision::Approve,
        )
        .await
        .unwrap();
}

const CREATE: &str = "/api/platform/orgs/org/apps/app/shared/tok1/records/notes/n1/comments";

#[tokio::test]
async fn cross_origin_redirect_does_not_carry_the_api_key() {
    let origin = MockServer::start().await;
    let other = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(CREATE))
        .respond_with(
            ResponseTemplate::new(307).insert_header("location", format!("{}/steal", other.uri())),
        )
        .mount(&origin)
        .await;
    Mock::given(wiremock::matchers::any())
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({"id": "c", "status": "pending"})),
        )
        .mount(&other)
        .await;
    let ip = "203.0.113.9".parse().unwrap();
    let _ = client(&origin)
        .create_shared_comment("org", "app", "tok1", "notes", "n1", None, "x", Some(ip))
        .await;
    assert!(other.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn same_origin_redirect_is_followed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(CREATE))
        .respond_with(ResponseTemplate::new(307).insert_header("location", "/moved"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/moved"))
        .and(header("x-api-key", "app-key"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({"id": "c", "status": "pending"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let ip = "203.0.113.9".parse().unwrap();
    client(&server)
        .create_shared_comment("org", "app", "tok1", "notes", "n1", None, "x", Some(ip))
        .await
        .unwrap();
}

#[tokio::test]
async fn transport_error_does_not_contain_the_token() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let c = CopepodClient::builder()
        .base_url(format!("http://{addr}"))
        .token("t")
        .auto_refresh(false)
        .build()
        .unwrap();
    let token = "SECRETtoken123";
    let err = c.shared_link_info("org", "app", token).await.unwrap_err();
    assert!(matches!(err, CopepodError::Http(_)), "{err:?}");
    assert!(!err.to_string().contains(token), "{err}");
    assert!(!format!("{err:?}").contains(token), "{err:?}");
}

#[tokio::test]
async fn decode_error_does_not_contain_the_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let token = "SECRETtoken123";
    let err = client(&server)
        .shared_link_info("org", "app", token)
        .await
        .unwrap_err();
    assert!(!err.to_string().contains(token), "{err}");
    assert!(!format!("{err:?}").contains(token), "{err:?}");
}

#[test]
fn tokens_are_redacted_in_debug() {
    let pair = TokenPair {
        token: "BEARER-SECRET".into(),
        refresh_token: "REFRESH-SECRET".into(),
        expires_at: None,
    };
    let text = format!("{pair:?}");
    assert!(
        !text.contains("BEARER-SECRET") && !text.contains("REFRESH-SECRET"),
        "{text}"
    );
    let c = CopepodClient::builder()
        .base_url("http://localhost:1")
        .token("BEARER-SECRET")
        .refresh_token("REFRESH-SECRET")
        .api_key("KEY-SECRET")
        .build()
        .unwrap();
    let text = format!("{c:?}");
    assert!(
        !text.contains("BEARER-SECRET")
            && !text.contains("REFRESH-SECRET")
            && !text.contains("KEY-SECRET"),
        "{text}"
    );
}

#[tokio::test]
async fn list_comments_status_only_approved_or_pending() {
    let server = MockServer::start().await;
    let c = client(&server);
    for bad in [
        SpaceCommentStatus::Rejected,
        SpaceCommentStatus::Removed,
        SpaceCommentStatus::Other,
    ] {
        let err = c
            .list_space_comments("org", "app", "sp1", None, Some(bad))
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{bad:?}");
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
