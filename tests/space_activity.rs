//! toolkit-ui4-ui5 S9: the Space activity log.

use copepod_sdk::{CopepodClient, CopepodError, SpaceActivityKind, SpaceMembership, SpaceRole};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const ACT: &str = "/api/platform/orgs/org/apps/app/spaces/sp1/activity";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn person(id: &str, name: serde_json::Value, membership: &str) -> serde_json::Value {
    json!({"user_id": id, "display_name": name, "membership": membership})
}

async fn page(c: &CopepodClient) -> copepod_sdk::SpaceActivityPage {
    c.list_space_activity("org", "app", "sp1", None, None, &[])
        .await
        .unwrap()
}

#[tokio::test]
async fn parses_every_kind_with_people_record_role_and_email() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .and(header("authorization", "Bearer user-bearer"))
        .and(|r: &Request| !r.headers.contains_key("x-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
              {"kind": "item.added", "created": "2026-11-18T10:00:00Z",
               "actor": person("u1", json!("Ana"), "member"),
               "record": {"collection": "notes", "record_id": "n1", "deleted": false}},
              {"kind": "item.deleted", "created": "2026-11-18T09:00:00Z",
               "actor": person("u1", json!("Ana"), "member"),
               "record": {"collection": "notes", "record_id": "n2", "deleted": true}},
              {"kind": "member.role_changed", "created": "2026-11-18T08:00:00Z",
               "actor": person("u1", json!("Ana"), "member"),
               "subject": person("u2", json!(null), "former"), "role": "editor"},
              {"kind": "invite.sent", "created": "2026-11-18T07:00:00Z",
               "actor": person("u1", json!(null), "member"), "invite_email": "b@example.com"}
            ],
            "next_before": "cursor-1"})))
        .expect(1)
        .mount(&server)
        .await;
    let p = page(&client(&server)).await;
    assert_eq!(p.next_before.as_deref(), Some("cursor-1"));
    assert_eq!(p.items.len(), 4);
    let a = &p.items[0];
    assert_eq!(a.kind, SpaceActivityKind::ItemAdded);
    assert_eq!(
        a.actor.as_ref().unwrap().display_name.as_deref(),
        Some("Ana")
    );
    assert_eq!(
        a.actor.as_ref().unwrap().membership,
        SpaceMembership::Member
    );
    assert_eq!(a.record.as_ref().unwrap().collection, "notes");
    assert!(!a.record.as_ref().unwrap().deleted);
    assert!(a.subject.is_none() && a.role.is_none() && a.invite_email.is_none());
    assert!(p.items[1].record.as_ref().unwrap().deleted);
    assert_eq!(p.items[1].kind, SpaceActivityKind::ItemDeleted);
    let r = &p.items[2];
    assert_eq!(r.kind, SpaceActivityKind::MemberRoleChanged);
    assert_eq!(r.role, Some(SpaceRole::Editor));
    let s = r.subject.as_ref().unwrap();
    assert_eq!(s.display_name, None);
    assert_eq!(s.membership, SpaceMembership::Former);
    assert_eq!(p.items[3].kind, SpaceActivityKind::InviteSent);
    assert_eq!(p.items[3].invite_email.as_deref(), Some("b@example.com"));
    assert_eq!(p.items[3].actor.as_ref().unwrap().display_name, None);
}

#[tokio::test]
async fn null_actor_comment_events_and_null_next_before() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
              {"kind": "comment.approved", "created": "2026-11-18T10:00:00Z", "actor": null,
               "record": {"collection": "notes", "record_id": "n1", "deleted": false}},
              {"kind": "comment.pending", "created": "2026-11-18T09:00:00Z", "actor": null,
               "record": {"collection": "notes", "record_id": "n1", "deleted": false}}],
            "next_before": null})))
        .mount(&server)
        .await;
    let p = page(&client(&server)).await;
    assert_eq!(p.items[0].kind, SpaceActivityKind::CommentApproved);
    assert_eq!(p.items[1].kind, SpaceActivityKind::CommentPending);
    assert!(p.items[0].actor.is_none());
    assert!(p.next_before.is_none());
}

#[tokio::test]
async fn unknown_kind_and_membership_still_deserialize() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
              {"kind": "space.archived", "created": "2026-11-18T10:00:00Z",
               "actor": person("u1", json!("Ana"), "guest"), "extra": 1},
              {"kind": "item.added", "created": "2026-11-18T09:00:00Z",
               "actor": person("u1", json!("Ana"), "member"),
               "record": {"collection": "notes", "record_id": "n1", "deleted": false}}],
            "next_before": null})))
        .mount(&server)
        .await;
    let p = page(&client(&server)).await;
    assert_eq!(p.items[0].kind, SpaceActivityKind::Other);
    assert_eq!(
        p.items[0].actor.as_ref().unwrap().membership,
        SpaceMembership::Other
    );
    assert_eq!(p.items[1].kind, SpaceActivityKind::ItemAdded);
}

#[tokio::test]
async fn query_params_sent_only_when_given() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .and(query_param("limit", "25"))
        .and(query_param("before", "cursor-1"))
        .and(query_param("collections", "notes,tasks"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"items": [], "next_before": null})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .and(|r: &Request| r.url.query().is_none())
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"items": [], "next_before": null})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    c.list_space_activity(
        "org",
        "app",
        "sp1",
        Some(25),
        Some("cursor-1"),
        &["notes", "tasks"],
    )
    .await
    .unwrap();
    c.list_space_activity("org", "app", "sp1", None, None, &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn before_cursor_is_url_encoded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .and(query_param("before", "a b&c=d/+"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"items": [], "next_before": null})),
        )
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .list_space_activity("org", "app", "sp1", None, Some("a b&c=d/+"), &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn errors_are_coded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/platform/orgs/org/apps/app/spaces/other/activity",
        ))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"code": "space_not_found", "message": "m"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(ACT))
        .and(query_param("before", "junk"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"code": "activity_cursor_invalid", "message": "m"})),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c
        .list_space_activity("org", "app", "other", None, None, &[])
        .await
        .unwrap_err();
    assert!(err.is_space_not_found(), "{err:?}");
    let err = c
        .list_space_activity("org", "app", "sp1", None, Some("junk"), &[])
        .await
        .unwrap_err();
    assert_eq!(err.api_status(), Some(400));
    assert_eq!(err.api_code(), Some("activity_cursor_invalid"));
}

#[tokio::test]
async fn rejects_bad_space_and_collection_names_before_any_call() {
    let server = MockServer::start().await;
    let c = client(&server);
    let err = c
        .list_space_activity("org", "app", "sp/1", None, None, &[])
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    for bad in ["no,tes", "a/b", ""] {
        let err = c
            .list_space_activity("org", "app", "sp1", None, None, &[bad])
            .await
            .unwrap_err();
        assert!(
            matches!(err, CopepodError::InvalidArgument(_)),
            "{bad:?}: {err:?}"
        );
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
