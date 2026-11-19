//! toolkit-ui4-ui5 S9: link info and visitor comments (public reads, API-key create).

use std::net::IpAddr;

use copepod_sdk::{
    CopepodClient, CopepodError, SharePermission, SharedLinkScope, SpaceCommentStatus,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const APP: &str = "/api/platform/orgs/org/apps/app";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("app-key")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn client_without_key(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .auto_refresh(false)
        .build()
        .unwrap()
}

/// Matches only requests carrying neither `Authorization` nor `X-API-Key`.
fn no_credentials(req: &Request) -> bool {
    !req.headers.contains_key("authorization") && !req.headers.contains_key("x-api-key")
}

fn coded(status: u16, code: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"}))
}

fn assert_coded(err: CopepodError, status: u16, code: &str) {
    assert!(
        matches!(&err, CopepodError::Api { status: s, code: Some(c), .. } if *s == status && c == code),
        "{err:?}"
    );
    assert_eq!(err.api_code(), Some(code));
}

// ---- shared_link_info ----

#[tokio::test]
async fn link_info_space_scope_sends_no_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/tok1")))
        .and(no_credentials)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "permission": "comment", "scope": {"kind": "space"},
            "expires_at": "2026-12-01T00:00:00Z"})))
        .expect(1)
        .mount(&server)
        .await;
    let info = client(&server)
        .shared_link_info("org", "app", "tok1")
        .await
        .unwrap();
    assert_eq!(info.permission, SharePermission::Comment);
    assert_eq!(info.scope, SharedLinkScope::Space);
    assert_eq!(info.expires_at, "2026-12-01T00:00:00Z");
}

#[tokio::test]
async fn link_info_record_scope_and_unknown_scope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/rec1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "permission": "read",
            "scope": {"kind": "record", "collection": "notes", "record_id": "n1"},
            "expires_at": "2026-12-01T00:00:00Z"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/future1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "permission": "read", "scope": {"kind": "folder"},
            "expires_at": "2026-12-01T00:00:00Z"})))
        .mount(&server)
        .await;
    let c = client(&server);
    let info = c.shared_link_info("org", "app", "rec1").await.unwrap();
    assert_eq!(info.permission, SharePermission::Read);
    assert_eq!(
        info.scope,
        SharedLinkScope::Record {
            collection: "notes".into(),
            record_id: "n1".into()
        }
    );
    let info = c.shared_link_info("org", "app", "future1").await.unwrap();
    assert_eq!(info.scope, SharedLinkScope::Other);
}

#[tokio::test]
async fn link_info_errors_are_coded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/gone")))
        .respond_with(coded(404, "share_link_not_found"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/limited")))
        .respond_with(coded(429, "link_rate_limited"))
        .mount(&server)
        .await;
    let c = client(&server);
    assert_coded(
        c.shared_link_info("org", "app", "gone").await.unwrap_err(),
        404,
        "share_link_not_found",
    );
    assert_coded(
        c.shared_link_info("org", "app", "limited")
            .await
            .unwrap_err(),
        429,
        "link_rate_limited",
    );
}

#[tokio::test]
async fn link_info_rejects_a_malformed_token_before_any_call() {
    let server = MockServer::start().await;
    let err = client(&server)
        .shared_link_info("org", "app", "a/../b")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

// ---- list_shared_comments ----

#[tokio::test]
async fn list_shared_comments_is_public_and_parses_null_author() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/tok1/records/notes/n1/comments")))
        .and(no_credentials)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [
            {"id": "c1", "author_name": "Ana", "body": "Nice", "created": "2026-11-18T10:00:00Z"},
            {"id": "c2", "author_name": null, "body": "Hi", "created": "2026-11-18T11:00:00Z"}]})))
        .expect(1)
        .mount(&server)
        .await;
    let items = client(&server)
        .list_shared_comments("org", "app", "tok1", "notes", "n1")
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, "c1");
    assert_eq!(items[0].author_name.as_deref(), Some("Ana"));
    assert_eq!(items[0].body, "Nice");
    assert_eq!(items[0].created, "2026-11-18T10:00:00Z");
    assert_eq!(items[1].author_name, None);
}

#[tokio::test]
async fn list_shared_comments_empty_and_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "{APP}/shared/tok1/records/notes/none/comments"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": []})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/dead/records/notes/n1/comments")))
        .respond_with(coded(404, "share_link_not_found"))
        .mount(&server)
        .await;
    let c = client(&server);
    assert!(c
        .list_shared_comments("org", "app", "tok1", "notes", "none")
        .await
        .unwrap()
        .is_empty());
    assert_coded(
        c.list_shared_comments("org", "app", "dead", "notes", "n1")
            .await
            .unwrap_err(),
        404,
        "share_link_not_found",
    );
}

#[tokio::test]
async fn list_shared_comments_rejects_bad_ids_before_any_call() {
    let server = MockServer::start().await;
    let c = client(&server);
    for (token, coll, id) in [
        ("tok1", "no/tes", "n1"),
        ("tok1", "notes", "n/1"),
        ("t k", "notes", "n1"),
    ] {
        let err = c
            .list_shared_comments("org", "app", token, coll, id)
            .await
            .unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

// ---- create_shared_comment ----

fn created_path(token: &str) -> String {
    format!("{APP}/shared/{token}/records/notes/n1/comments")
}

#[tokio::test]
async fn create_with_api_key_sends_key_not_bearer_and_client_ip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(created_path("tok1")))
        .and(header("x-api-key", "app-key"))
        .and(|r: &Request| !r.headers.contains_key("authorization"))
        .and(body_json(
            json!({"author_name": "Ana", "body": "Nice note", "client_ip": "203.0.113.9"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "c9", "status": "pending",
            "notify": {"space_id": "sp1", "collection": "notes", "record_id": "n1",
                "owner_user_ids": ["u1", "u2"], "first_in_hour": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let ip: IpAddr = "203.0.113.9".parse().unwrap();
    let out = client(&server)
        .create_shared_comment(
            "org",
            "app",
            "tok1",
            "notes",
            "n1",
            Some("Ana"),
            "Nice note",
            Some(ip),
        )
        .await
        .unwrap();
    assert_eq!(out.id, "c9");
    assert_eq!(out.status, SpaceCommentStatus::Pending);
    let notify = out.notify.unwrap();
    assert_eq!(notify.space_id, "sp1");
    assert_eq!(notify.collection, "notes");
    assert_eq!(notify.record_id, "n1");
    assert_eq!(notify.owner_user_ids, vec!["u1", "u2"]);
    assert!(notify.first_in_hour);
}

#[tokio::test]
async fn create_accepts_an_ipv6_client_ip() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(created_path("tok1")))
        .and(body_json(json!({"body": "x", "client_ip": "2001:db8::1"})))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({"id": "c1", "status": "pending"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let ip: IpAddr = "2001:db8::1".parse().unwrap();
    client(&server)
        .create_shared_comment("org", "app", "tok1", "notes", "n1", None, "x", Some(ip))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_without_key_or_ip_sends_no_credentials_and_omits_optionals() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(created_path("tok1")))
        .and(no_credentials)
        .and(body_json(json!({"body": "Hello"})))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({"id": "c1", "status": "pending"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let out = client_without_key(&server)
        .create_shared_comment("org", "app", "tok1", "notes", "n1", None, "Hello", None)
        .await
        .unwrap();
    assert_eq!(out.id, "c1");
    assert!(out.notify.is_none());
}

#[tokio::test]
async fn create_errors_are_coded() {
    let server = MockServer::start().await;
    for (token, status, code) in [
        ("ro", 403, "link_read_only"),
        ("gone", 404, "share_link_not_found"),
        ("bad", 422, "comment_invalid"),
        ("rate", 429, "comment_rate_limited"),
        ("full", 429, "comment_queue_full"),
        ("limited", 429, "link_rate_limited"),
    ] {
        Mock::given(method("POST"))
            .and(path(created_path(token)))
            .respond_with(coded(status, code))
            .mount(&server)
            .await;
    }
    let c = client(&server);
    for (token, status, code) in [
        ("ro", 403, "link_read_only"),
        ("gone", 404, "share_link_not_found"),
        ("bad", 422, "comment_invalid"),
        ("rate", 429, "comment_rate_limited"),
        ("full", 429, "comment_queue_full"),
        ("limited", 429, "link_rate_limited"),
    ] {
        let err = c
            .create_shared_comment("org", "app", token, "notes", "n1", None, "x", None)
            .await
            .unwrap_err();
        assert_coded(err, status, code);
    }
}

#[tokio::test]
async fn create_rejects_bad_ids_before_any_call() {
    let server = MockServer::start().await;
    let c = client(&server);
    let err = c
        .create_shared_comment("org", "app", "tok/1", "notes", "n1", None, "x", None)
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    let err = c
        .create_shared_comment("org", "app", "tok1", "notes", "n/1", None, "x", None)
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}
