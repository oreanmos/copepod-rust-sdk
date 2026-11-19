//! toolkit-ui4-ui5 S9: member comment list and the owner's decision (bearer).

use copepod_sdk::{CommentDecision, CopepodClient, CopepodError, SpaceCommentStatus};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SP: &str = "/api/platform/orgs/org/apps/app/spaces/sp1";

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

fn coded(status: u16, code: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"}))
}

#[tokio::test]
async fn list_sends_bearer_and_query_and_parses_items() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{SP}/comments")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(no_api_key)
        .and(query_param("record_id", "n1"))
        .and(query_param("status", "pending"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [
            {"id": "c1", "collection": "notes", "record_id": "n1", "author_name": null,
             "body": "Hi", "status": "pending", "created": "2026-11-18T10:00:00Z", "decided_at": null},
            {"id": "c2", "collection": "notes", "record_id": "n1", "author_name": "Ana",
             "body": "Ok", "status": "approved", "created": "2026-11-18T11:00:00Z",
             "decided_at": "2026-11-18T12:00:00Z"}]})))
        .expect(1)
        .mount(&server)
        .await;
    let items = client(&server)
        .list_space_comments(
            "org",
            "app",
            "sp1",
            Some("n1"),
            Some(SpaceCommentStatus::Pending),
        )
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].status, SpaceCommentStatus::Pending);
    assert_eq!(items[0].author_name, None);
    assert_eq!(items[0].decided_at, None);
    assert_eq!(items[0].collection, "notes");
    assert_eq!(items[0].record_id, "n1");
    assert_eq!(items[1].status, SpaceCommentStatus::Approved);
    assert_eq!(items[1].author_name.as_deref(), Some("Ana"));
    assert_eq!(items[1].decided_at.as_deref(), Some("2026-11-18T12:00:00Z"));
}

#[tokio::test]
async fn list_without_filters_sends_no_query_and_unknown_status_parses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{SP}/comments")))
        .and(|r: &Request| r.url.query().is_none())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [
            {"id": "c1", "collection": "notes", "record_id": "n1", "author_name": null,
             "body": "Hi", "status": "archived", "created": "2026-11-18T10:00:00Z", "decided_at": null}]})))
        .expect(1)
        .mount(&server)
        .await;
    let items = client(&server)
        .list_space_comments("org", "app", "sp1", None, None)
        .await
        .unwrap();
    assert_eq!(items[0].status, SpaceCommentStatus::Other);
}

#[tokio::test]
async fn list_errors_are_coded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{SP}/comments")))
        .and(query_param("status", "pending"))
        .respond_with(coded(403, "insufficient_role"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/platform/orgs/org/apps/app/spaces/other/comments",
        ))
        .respond_with(coded(404, "space_not_found"))
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c
        .list_space_comments("org", "app", "sp1", None, Some(SpaceCommentStatus::Pending))
        .await
        .unwrap_err();
    assert!(err.is_insufficient_role(), "{err:?}");
    let err = c
        .list_space_comments("org", "app", "other", None, None)
        .await
        .unwrap_err();
    assert!(err.is_space_not_found(), "{err:?}");
}

#[tokio::test]
async fn list_rejects_bad_ids_before_any_call() {
    let server = MockServer::start().await;
    let c = client(&server);
    let err = c
        .list_space_comments("org", "app", "sp/1", None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    let err = c
        .list_space_comments("org", "app", "sp1", Some("a&b/c"), None)
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn decide_posts_each_decision_with_bearer() {
    let server = MockServer::start().await;
    for (decision, wire, status) in [
        (CommentDecision::Approve, "approve", "approved"),
        (CommentDecision::Reject, "reject", "rejected"),
        (CommentDecision::Remove, "remove", "removed"),
    ] {
        server.reset().await;
        Mock::given(method("POST"))
            .and(path(format!("{SP}/comments/c1/decision")))
            .and(header("authorization", "Bearer user-bearer"))
            .and(no_api_key)
            .and(body_json(json!({"decision": wire})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"id": "c1", "status": status})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let out = client(&server)
            .decide_space_comment("org", "app", "sp1", "c1", decision)
            .await
            .unwrap();
        assert_eq!(out.id, "c1");
        let want = match decision {
            CommentDecision::Approve => SpaceCommentStatus::Approved,
            CommentDecision::Reject => SpaceCommentStatus::Rejected,
            CommentDecision::Remove => SpaceCommentStatus::Removed,
        };
        assert_eq!(out.status, want);
    }
}

#[tokio::test]
async fn decide_errors_are_coded() {
    let server = MockServer::start().await;
    for (id, status, code) in [("notowner", 403, "insufficient_role")] {
        Mock::given(method("POST"))
            .and(path(format!("{SP}/comments/{id}/decision")))
            .respond_with(coded(status, code))
            .mount(&server)
            .await;
    }
    // The server answers an already-decided comment with a plain, uncoded 409.
    Mock::given(method("POST"))
        .and(path(format!("{SP}/comments/decided/decision")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"message": "m"})))
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c
        .decide_space_comment("org", "app", "sp1", "notowner", CommentDecision::Approve)
        .await
        .unwrap_err();
    assert!(err.is_insufficient_role(), "{err:?}");
    let err = c
        .decide_space_comment("org", "app", "sp1", "decided", CommentDecision::Reject)
        .await
        .unwrap_err();
    assert_eq!(err.api_status(), Some(409), "{err:?}");
    assert_eq!(err.api_code(), None, "{err:?}");
}

#[tokio::test]
async fn decide_rejects_bad_ids_before_any_call() {
    let server = MockServer::start().await;
    let err = client(&server)
        .decide_space_comment("org", "app", "sp1", "c/1", CommentDecision::Approve)
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}
