//! toolkit-ui4-ui5 S9: the app user's own display name.

use copepod_sdk::{CopepodClient, CopepodError};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const PROFILE: &str = "/api/platform/orgs/org/apps/app/auth/users/me/profile";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn no_api_key(req: &Request) -> bool {
    !req.headers.contains_key("x-api-key")
}

#[tokio::test]
async fn get_sends_bearer_and_parses_name_and_null() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(PROFILE))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": "Ana"})))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(PROFILE))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": null})))
        .mount(&server)
        .await;
    let c = client(&server);
    assert_eq!(
        c.get_my_profile("org", "app", "users")
            .await
            .unwrap()
            .name
            .as_deref(),
        Some("Ana")
    );
    assert_eq!(
        c.get_my_profile("org", "app", "users").await.unwrap().name,
        None
    );
}

#[tokio::test]
async fn update_patches_name_and_null_clears() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(PROFILE))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .and(body_json(json!({"name": "Ana Silva"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": "Ana Silva"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(PROFILE))
        .and(body_json(json!({"name": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": null})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let out = c
        .update_my_profile("org", "app", "users", Some("Ana Silva"))
        .await
        .unwrap();
    assert_eq!(out.name.as_deref(), Some("Ana Silva"));
    let out = c
        .update_my_profile("org", "app", "users", None)
        .await
        .unwrap();
    assert_eq!(out.name, None);
}

#[tokio::test]
async fn errors_are_coded() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(PROFILE))
        .respond_with(
            ResponseTemplate::new(422)
                .set_body_json(json!({"code": "name_invalid", "message": "m"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/platform/orgs/org/apps/app/auth/other/me/profile",
        ))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(json!({"code": "not_found", "message": "m"})),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c
        .update_my_profile("org", "app", "users", Some("x\u{202e}"))
        .await
        .unwrap_err();
    assert!(
        matches!(&err, CopepodError::Api { status: 422, code: Some(c), .. } if c == "name_invalid"),
        "{err:?}"
    );
    assert_eq!(err.api_code(), Some("name_invalid"));
    let err = c.get_my_profile("org", "app", "other").await.unwrap_err();
    assert_eq!(err.api_status(), Some(404));
}

#[tokio::test]
async fn rejects_a_bad_collection_before_any_call() {
    let server = MockServer::start().await;
    let c = client(&server);
    for bad in ["", "a/b", "..", "a?b"] {
        let err = c.get_my_profile("org", "app", bad).await.unwrap_err();
        assert!(
            matches!(err, CopepodError::InvalidArgument(_)),
            "{bad:?}: {err:?}"
        );
        let err = c
            .update_my_profile("org", "app", bad, None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, CopepodError::InvalidArgument(_)),
            "{bad:?}: {err:?}"
        );
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
