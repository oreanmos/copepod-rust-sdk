use copepod_sdk::error::*;
use copepod_sdk::{
    CopepodClient, CopepodError, ShareLinkCreate, ShareLinkScope, SharePermission,
    SharedRecordsQuery, SpaceKind, SpaceView, SuggestionCreate, SuggestionListOptions,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const APP: &str = "/api/platform/orgs/org/apps/app";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn link(id: &str, scope: serde_json::Value) -> serde_json::Value {
    json!({"id": id, "space_id": "sp1", "scope": scope, "permission": "read",
        "created_by": "u1", "expires_at": "2026-11-01T00:00:00Z",
        "revoked_at": null, "created": "2026-10-01T00:00:00Z"})
}

fn issued(id: &str, scope: serde_json::Value) -> serde_json::Value {
    json!({"token": "tok123", "url_path": format!("{APP}/shared/tok123"), "link": link(id, scope)})
}

fn suggestion(status: &str) -> serde_json::Value {
    json!({"id": "sg1", "space_id": "sp1", "collection": "notes", "record_id": "",
        "proposed": {"title": "x"}, "reason": "typo", "submitted_by": "u2",
        "status": status, "decided_by": "", "decided_at": null, "decision_note": "",
        "applied_record_id": "", "base_rev": "", "applied_at": null,
        "created": "2026-10-01T00:00:00Z"})
}

async fn err_of<T: std::fmt::Debug>(r: Result<T>) -> CopepodError {
    r.unwrap_err()
}

#[tokio::test]
async fn visitor_shape_of_get_space() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/c1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"id": "c1", "name": "Wiki", "kind": "commons"})),
        )
        .expect(2)
        .mount(&server)
        .await;
    let c = client(&server);
    match c.get_space_view("org", "app", "c1").await.unwrap() {
        SpaceView::Visitor(v) => {
            assert_eq!((v.id.as_str(), v.name.as_str()), ("c1", "Wiki"));
            assert_eq!(v.kind, SpaceKind::Commons);
        }
        other => panic!("{other:?}"),
    }
    assert!(c.get_space("org", "app", "c1").await.is_err());
}

#[tokio::test]
async fn member_shape_through_get_space_view() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/sp1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "sp1", "app_id": "app", "name": "Home", "kind": "private",
            "owner_user_id": "u1", "anonymous_read": false,
            "created": "2026-10-01T00:00:00Z", "role": "owner"})))
        .mount(&server)
        .await;
    let view = client(&server)
        .get_space_view("org", "app", "sp1")
        .await
        .unwrap();
    assert!(matches!(view, SpaceView::Member(m) if m.space.id == "sp1"));
}

#[tokio::test]
async fn share_link_create_list_revoke_rotate() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/sp1/share-links")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(json!({"scope": "space", "permission": "read"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(issued("l1", json!("space"))))
        .expect(1)
        .mount(&server)
        .await;
    let rec_scope = json!({"record_collection": "notes", "record_id": "r1"});
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/sp1/share-links")))
        .and(body_json(
            json!({"scope": rec_scope, "permission": "comment",
            "expires_at": "2026-12-01T00:00:00Z"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(issued("l2", rec_scope.clone())))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/sp1/share-links")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"items": [link("l1", json!("space"))]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{APP}/share-links/l1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"revoked": true})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/share-links/l1/rotate")))
        .and(body_json(json!({"expires_at": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(issued("l3", json!("space"))))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/share-links/l2/rotate")))
        .and(body_json(json!({"expires_at": "2027-01-01T00:00:00Z"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(issued("l4", rec_scope.clone())))
        .expect(1)
        .mount(&server)
        .await;

    let c = client(&server);
    let made = c
        .create_share_link(
            "org",
            "app",
            "sp1",
            &ShareLinkCreate {
                scope: ShareLinkScope::whole_space(),
                permission: SharePermission::Read,
                expires_at: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(made.token, "tok123");
    assert_eq!(made.link.scope, ShareLinkScope::whole_space());
    let rec = c
        .create_share_link(
            "org",
            "app",
            "sp1",
            &ShareLinkCreate {
                scope: ShareLinkScope::record("notes", "r1"),
                permission: SharePermission::Comment,
                expires_at: Some("2026-12-01T00:00:00Z".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(rec.link.scope, ShareLinkScope::record("notes", "r1"));
    let listed = c.list_share_links("org", "app", "sp1").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].revoked_at, None);
    c.revoke_share_link("org", "app", "l1").await.unwrap();
    assert_eq!(
        c.rotate_share_link("org", "app", "l1", None)
            .await
            .unwrap()
            .link
            .id,
        "l3"
    );
    assert_eq!(
        c.rotate_share_link("org", "app", "l2", Some("2027-01-01T00:00:00Z"))
            .await
            .unwrap()
            .link
            .id,
        "l4"
    );
}

#[tokio::test]
async fn share_link_errors_keep_their_codes() {
    let server = MockServer::start().await;
    for (verb, p, status, code) in [
        (
            "POST",
            "/spaces/sp1/share-links",
            503,
            SHARE_LINKS_UPGRADING_CODE,
        ),
        (
            "GET",
            "/spaces/sp1/share-links",
            403,
            INSUFFICIENT_ROLE_CODE,
        ),
        ("DELETE", "/share-links/l1", 404, LINK_NOT_FOUND_CODE),
        ("POST", "/share-links/l1/rotate", 410, LINK_REVOKED_CODE),
        (
            "POST",
            "/share-links/l2/rotate",
            429,
            SHARE_LINK_RATE_LIMITED_CODE,
        ),
    ] {
        Mock::given(method(verb))
            .and(path(format!("{APP}{p}")))
            .respond_with(
                ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"})),
            )
            .mount(&server)
            .await;
    }
    let c = client(&server);
    let body = ShareLinkCreate {
        scope: ShareLinkScope::whole_space(),
        permission: SharePermission::Read,
        expires_at: None,
    };
    let e = err_of(c.create_share_link("org", "app", "sp1", &body).await).await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(503), Some("share_links_upgrading"))
    );
    assert!(err_of(c.list_share_links("org", "app", "sp1").await)
        .await
        .is_insufficient_role());
    let e = err_of(c.revoke_share_link("org", "app", "l1").await).await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(404), Some("link_not_found"))
    );
    let e = err_of(c.rotate_share_link("org", "app", "l1", None).await).await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(410), Some("link_revoked"))
    );
    let e = err_of(c.rotate_share_link("org", "app", "l2", None).await).await;
    assert_eq!(e.api_code(), Some("share_link_rate_limited"));
}

struct NoCredentials;
impl Respond for NoCredentials {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let leaked =
            req.headers.contains_key("authorization") || req.headers.contains_key("x-api-key");
        if leaked {
            ResponseTemplate::new(400).set_body_json(json!({"code": "leaked", "message": "creds"}))
        } else {
            ResponseTemplate::new(200).set_body_json(json!({
                "page": 2, "per_page": 10, "total_items": 1, "total_pages": 1,
                "items": [{"id": "r1", "title": "t"}]}))
        }
    }
}

#[tokio::test]
async fn shared_reads_send_no_credentials_and_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/tok123/records/notes")))
        .and(query_param("filter", "title='t'"))
        .and(query_param("sort", "-created"))
        .and(query_param("fields", "id,title"))
        .and(query_param("page", "2"))
        .and(query_param("per_page", "10"))
        .respond_with(NoCredentials)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/tok123/records/notes/r1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "r1"})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let page = c
        .list_shared_records(
            "org",
            "app",
            "tok123",
            "notes",
            &SharedRecordsQuery {
                filter: Some("title='t'".into()),
                sort: Some("-created".into()),
                fields: Some("id,title".into()),
                page: Some(2),
                per_page: Some(10),
            },
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.page, 2);
    let one = c
        .get_shared_record("org", "app", "tok123", "notes", "r1")
        .await
        .unwrap();
    assert_eq!(one["id"], "r1");
    for req in server.received_requests().await.unwrap() {
        assert!(!req.headers.contains_key("authorization"));
        assert!(!req.headers.contains_key("x-api-key"));
    }
}

#[tokio::test]
async fn shared_read_errors_keep_their_codes() {
    let server = MockServer::start().await;
    for (tok, status, code) in [
        ("gone", 410, LINK_REVOKED_CODE),
        ("old", 410, LINK_EXPIRED_CODE),
        ("spam", 429, LINK_RATE_LIMITED_CODE),
    ] {
        Mock::given(method("GET"))
            .and(path(format!("{APP}/shared/{tok}/records/notes")))
            .respond_with(
                ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"})),
            )
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path(format!("{APP}/shared/nope/records/notes/r1")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "not found"})))
        .mount(&server)
        .await;
    let c = client(&server);
    let q = SharedRecordsQuery::default();
    for (tok, status, code) in [
        ("gone", 410, "link_revoked"),
        ("old", 410, "link_expired"),
        ("spam", 429, "link_rate_limited"),
    ] {
        let e = err_of(c.list_shared_records("org", "app", tok, "notes", &q).await).await;
        assert_eq!((e.api_status(), e.api_code()), (Some(status), Some(code)));
    }
    let e = err_of(
        c.get_shared_record("org", "app", "nope", "notes", "r1")
            .await,
    )
    .await;
    assert_eq!(e.api_status(), Some(404));
}

#[tokio::test]
async fn suggestions_create_list_accept_reject() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/suggestions")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(
            json!({"collection": "notes", "proposed": {"title": "x"}, "reason": "typo"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(suggestion("pending")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/suggestions")))
        .and(body_json(json!({"collection": "notes", "record_id": "r9", "proposed": {"title": "y"}, "reason": "fix"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(suggestion("pending")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/c1/suggestions")))
        .and(query_param("status", "pending"))
        .and(query_param("limit", "1"))
        .and(query_param("after", "sg0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"items": [suggestion("pending")], "next": "sg1"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/c1/suggestions")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [], "next": null})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/suggestions/sg1/accept")))
        .and(body_json(json!({"note": "thanks"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "suggestion": suggestion("accepted"), "record": {"id": "r1", "title": "x"}})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/suggestions/sg1/reject")))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(suggestion("rejected")))
        .expect(1)
        .mount(&server)
        .await;

    let c = client(&server);
    let new = SuggestionCreate {
        collection: "notes".into(),
        record_id: None,
        proposed: json!({"title": "x"}),
        reason: "typo".into(),
    };
    assert_eq!(
        c.create_space_suggestion("org", "app", "c1", &new)
            .await
            .unwrap()
            .status,
        "pending"
    );
    let change = SuggestionCreate {
        record_id: Some("r9".into()),
        proposed: json!({"title": "y"}),
        reason: "fix".into(),
        ..new.clone()
    };
    c.create_space_suggestion("org", "app", "c1", &change)
        .await
        .unwrap();
    let page = c
        .list_space_suggestions(
            "org",
            "app",
            "c1",
            &SuggestionListOptions {
                status: Some("pending".into()),
                limit: Some(1),
                after: Some("sg0".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(page.next.as_deref(), Some("sg1"));
    let last = c
        .list_space_suggestions("org", "app", "c1", &Default::default())
        .await
        .unwrap();
    assert!(last.items.is_empty() && last.next.is_none());
    let done = c
        .accept_space_suggestion("org", "app", "c1", "sg1", Some("thanks"))
        .await
        .unwrap();
    assert_eq!(done.suggestion.status, "accepted");
    assert_eq!(done.record["id"], "r1");
    let rej = c
        .reject_space_suggestion("org", "app", "c1", "sg1", None)
        .await
        .unwrap();
    assert_eq!(rej.status, "rejected");
}

#[tokio::test]
async fn suggestion_errors_keep_their_codes() {
    let server = MockServer::start().await;
    let cases = [
        ("POST", "suggestions", 429, SUGGESTION_RATE_LIMITED_CODE),
        ("GET", "suggestions", 400, NOT_COMMONS_CODE),
        (
            "POST",
            "suggestions/sg1/reject",
            409,
            SUGGESTION_DECIDED_CODE,
        ),
        (
            "POST",
            "suggestions/sg2/accept",
            404,
            SUGGESTION_NOT_FOUND_CODE,
        ),
        (
            "POST",
            "suggestions/sg3/accept",
            503,
            COMMONS_UPGRADING_CODE,
        ),
        (
            "POST",
            "records/notes/r1/takedown",
            403,
            INSUFFICIENT_ROLE_CODE,
        ),
    ];
    for (verb, p, status, code) in cases {
        Mock::given(method(verb))
            .and(path(format!("{APP}/spaces/c1/{p}")))
            .respond_with(
                ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"})),
            )
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/suggestions/sg4/accept")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "revision_conflict", "message": "m", "current_rev": 7})))
        .mount(&server)
        .await;
    let c = client(&server);
    let new = SuggestionCreate {
        collection: "notes".into(),
        record_id: None,
        proposed: json!({"a": 1}),
        reason: "r".into(),
    };
    let e = err_of(c.create_space_suggestion("org", "app", "c1", &new).await).await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(429), Some("suggestion_rate_limited"))
    );
    let e = err_of(
        c.list_space_suggestions("org", "app", "c1", &Default::default())
            .await,
    )
    .await;
    assert_eq!(e.api_code(), Some("not_commons"));
    let e = err_of(
        c.reject_space_suggestion("org", "app", "c1", "sg1", None)
            .await,
    )
    .await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(409), Some("suggestion_decided"))
    );
    let e = err_of(
        c.accept_space_suggestion("org", "app", "c1", "sg2", None)
            .await,
    )
    .await;
    assert_eq!(e.api_code(), Some("suggestion_not_found"));
    let e = err_of(
        c.accept_space_suggestion("org", "app", "c1", "sg3", None)
            .await,
    )
    .await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(503), Some("commons_upgrading"))
    );
    let e = err_of(
        c.accept_space_suggestion("org", "app", "c1", "sg4", None)
            .await,
    )
    .await;
    assert!(e.is_revision_conflict());
    assert_eq!(e.revision_conflict_details().unwrap().current_rev, json!(7));
    let e = err_of(
        c.takedown_space_record("org", "app", "c1", "notes", "r1", "spam")
            .await,
    )
    .await;
    assert!(e.is_insufficient_role());
}

#[tokio::test]
async fn takedown_posts_the_reason() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/records/notes/r1/takedown")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(json!({"reason": "spam"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"taken_down": true, "record_id": "r1", "created_by": null})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let out = client(&server)
        .takedown_space_record("org", "app", "c1", "notes", "r1", "spam")
        .await
        .unwrap();
    assert!(out.taken_down);
    assert_eq!(out.record_id, "r1");
    assert_eq!(out.created_by, None);
}

#[tokio::test]
async fn management_calls_never_send_the_api_key_and_bad_arguments_fail_early() {
    let server = MockServer::start().await;
    Mock::given(header_exists("x-api-key"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let c = client(&server);
    let sugg = |reason: &str, proposed: serde_json::Value| SuggestionCreate {
        collection: "notes".into(),
        record_id: None,
        proposed,
        reason: reason.into(),
    };
    let link = |expires: Option<&str>| ShareLinkCreate {
        scope: ShareLinkScope::whole_space(),
        permission: SharePermission::Read,
        expires_at: expires.map(String::from),
    };
    let big = json!({"t": "x".repeat(17 * 1024)});
    assert!(matches!(
        c.create_space_suggestion("o", "a", "c1", &sugg("", json!({})))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.create_space_suggestion("o", "a", "c1", &sugg("r", json!([])))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.create_space_suggestion("o", "a", "c1", &sugg("r", big))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.create_space_suggestion("o", "a", "a/b", &sugg("r", json!({})))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    let opts = |status: Option<&str>, limit| SuggestionListOptions {
        status: status.map(String::from),
        limit,
        after: None,
    };
    assert!(matches!(
        c.list_space_suggestions("o", "a", "c1", &opts(Some("done"), None))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.list_space_suggestions("o", "a", "c1", &opts(None, Some(0)))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.list_space_suggestions("o", "a", "c1", &opts(None, Some(201)))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.accept_space_suggestion("o", "a", "c1", "", None).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.reject_space_suggestion("o", "a", "c1", "s", Some(&"n".repeat(1001)))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.takedown_space_record("o", "a", "c1", "notes", "r1", "  ")
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.create_share_link("o", "a", "sp", &link(Some("tomorrow")))
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.rotate_share_link("o", "a", "l1", Some("bad")).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.revoke_share_link("o", "a", "").await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.get_shared_record("o", "a", &"t".repeat(257), "notes", "r1")
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.list_shared_records("o", "a", "t/x", "notes", &Default::default())
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[test]
fn error_code_constants_match_the_server() {
    assert_eq!(SPACE_RATE_LIMITED_CODE, "space_rate_limited");
    assert_eq!(SUGGESTION_RATE_LIMITED_CODE, "suggestion_rate_limited");
    assert_eq!(SUGGESTION_DECIDED_CODE, "suggestion_decided");
    assert_eq!(REVISION_CONFLICT_CODE, "revision_conflict");
    assert_eq!(SHARE_LINKS_UPGRADING_CODE, "share_links_upgrading");
    assert_eq!(COMMONS_INCOMPLETE_CODE, "commons_incomplete");
    assert_eq!(LINK_REVOKED_CODE, "link_revoked");
    assert_eq!(NOT_COMMONS_CODE, "not_commons");
    assert_eq!(INSUFFICIENT_ROLE_CODE, "insufficient_role");
}
