use copepod_sdk::*;
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const AUTH: &str = "/api/platform/orgs/org/apps/app/auth/users";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("stale-bearer")
        .api_key("key-1")
        .auto_refresh(false)
        .build()
        .unwrap()
}

async fn mount(server: &MockServer, route: &str, status: u16, body: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path(format!("{AUTH}/{route}")))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(server)
        .await;
}

fn suspended(with_token: bool) -> serde_json::Value {
    let mut b = json!({"status":403,"code":"account_suspended","error":"account_suspended",
                       "message":"This account is suspended."});
    if with_token {
        b["erasure_token"] = json!("tok-1");
        b["erasure_token_expires_in"] = json!(600);
    }
    b
}

#[tokio::test]
async fn login_suspended_carries_erasure_token() {
    let server = MockServer::start().await;
    mount(&server, "auth-with-password", 403, suspended(true)).await;
    let err = client(&server)
        .app("org", "app")
        .auth("users")
        .login("a@example.invalid", "pw")
        .await
        .unwrap_err();
    assert!(err.is_account_suspended());
    let d = err.account_suspended_details().unwrap();
    assert_eq!(d.erasure_token.as_deref(), Some("tok-1"));
    assert_eq!(d.expires_in, Some(600));
}

#[tokio::test]
async fn mfa_verify_suspended_carries_erasure_token() {
    let server = MockServer::start().await;
    mount(&server, "mfa/verify", 403, suspended(true)).await;
    let err = client(&server)
        .app("org", "app")
        .auth("users")
        .mfa_verify("m", "123456")
        .await
        .unwrap_err();
    assert_eq!(
        err.account_suspended_details()
            .unwrap()
            .erasure_token
            .as_deref(),
        Some("tok-1")
    );
}

#[tokio::test]
async fn suspended_without_token_still_suspended() {
    let server = MockServer::start().await;
    mount(&server, "auth-with-password", 403, suspended(false)).await;
    let err = client(&server)
        .app("org", "app")
        .auth("users")
        .login("a", "b")
        .await
        .unwrap_err();
    assert!(err.is_account_suspended());
    let d = err.account_suspended_details().unwrap();
    assert_eq!(d.erasure_token, None);
    assert_eq!(d.expires_in, None);
}

#[tokio::test]
async fn plain_api_suspended_has_no_token() {
    let err = CopepodError::Api {
        status: 403,
        code: Some("account_suspended".into()),
        message: "x".into(),
    };
    assert!(err.is_account_suspended());
    assert_eq!(
        err.account_suspended_details(),
        Some(AccountSuspended::default())
    );
    let other = CopepodError::Api {
        status: 403,
        code: Some("nope".into()),
        message: "x".into(),
    };
    assert!(other.account_suspended_details().is_none());
}

#[tokio::test]
async fn erase_success_sends_no_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{AUTH}/erase")))
        .and(body_json(json!({"erasure_token":"tok-1","confirm":true})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"deleted":true,"erased_rows":7})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let out = client(&server)
        .app("org", "app")
        .auth("users")
        .erase_account("tok-1")
        .await
        .unwrap();
    assert!(out.deleted);
    assert_eq!(out.erased_rows, 7);
    let r = &server.received_requests().await.unwrap()[0];
    assert!(
        r.headers.get("authorization").is_none(),
        "erase sent Authorization"
    );
    assert!(r.headers.get("x-api-key").is_none(), "erase sent X-API-Key");
}

#[tokio::test]
async fn erase_bad_token_is_401() {
    let server = MockServer::start().await;
    mount(
        &server,
        "erase",
        401,
        json!({"status":401,"code":"unauthorized","message":"Unauthorized"}),
    )
    .await;
    let err = client(&server)
        .app_erase_account("org", "app", "users", "bad")
        .await
        .unwrap_err();
    assert!(err.is_erasure_token_invalid());
}

#[tokio::test]
async fn erase_billing_failure_then_retry_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{AUTH}/erase")))
        .and(body_json(json!({"erasure_token":"tok-1","confirm":true})))
        .respond_with(ResponseTemplate::new(503).set_body_json(
            json!({"status":503,"code":"billing_cancellation_failed","message":"try again"}),
        ))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{AUTH}/erase")))
        .and(body_json(json!({"erasure_token":"tok-2","confirm":true})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"deleted":true,"erased_rows":1})),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c
        .app_erase_account("org", "app", "users", "tok-1")
        .await
        .unwrap_err();
    assert!(err.is_billing_cancellation_failed());
    assert!(!err.is_trust_upgrading());
    assert!(
        c.app_erase_account("org", "app", "users", "tok-2")
            .await
            .unwrap()
            .deleted
    );
}

#[tokio::test]
async fn erase_trust_upgrading_and_empty_token() {
    let server = MockServer::start().await;
    mount(
        &server,
        "erase",
        503,
        json!({"status":503,"code":"trust_upgrading","message":"up"}),
    )
    .await;
    let c = client(&server);
    assert!(c
        .app_erase_account("org", "app", "users", "t")
        .await
        .unwrap_err()
        .is_trust_upgrading());
    let err = c
        .app_erase_account("org", "app", "users", " ")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn signup_unavailable_is_recognised() {
    let server = MockServer::start().await;
    mount(
        &server,
        "register",
        403,
        json!({"status":403,"code":"signup_unavailable","message":"This email address can't be used to sign up right now."}),
    )
    .await;
    let err = client(&server)
        .app_register(
            "org",
            "app",
            "users",
            &json!({"email":"a@example.invalid","password":"pw"}),
        )
        .await
        .unwrap_err();
    assert!(err.is_signup_unavailable());
    assert!(!err.is_account_suspended());
}
