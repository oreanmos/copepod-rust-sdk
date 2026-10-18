use copepod_sdk::{
    CopepodClient, CopepodError, RealtimeSubscriptionOptions, RecordUpdateOptions, SpaceCreate,
    SpaceInviteCreate, SpaceKind, SpaceRole,
};
use futures_util::StreamExt;
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

fn space(id: &str) -> serde_json::Value {
    json!({"id": id, "app_id": "app", "name": "Home", "kind": "private",
        "owner_user_id": "u1", "anonymous_read": false,
        "created": "2026-10-01T00:00:00Z", "deleted_at": null})
}

#[tokio::test]
async fn space_crud_uses_bearer_and_documented_shapes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(json!({"name": "Home", "kind": "private"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(space("sp1")))
        .expect(1)
        .mount(&server)
        .await;
    let mut listed = space("sp1");
    listed["role"] = json!("editor");
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [listed.clone()]})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/sp1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(listed))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{APP}/spaces/sp1")))
        .and(body_json(json!({"name": "Renamed"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(space("sp1")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{APP}/spaces/sp1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted": true})))
        .expect(1)
        .mount(&server)
        .await;

    let c = client(&server);
    let created = c
        .create_space(
            "org",
            "app",
            &SpaceCreate {
                name: "Home".into(),
                kind: SpaceKind::Private,
            },
        )
        .await
        .unwrap();
    assert_eq!(created.id, "sp1");
    assert_eq!(created.kind, SpaceKind::Private);
    let spaces = c.list_spaces("org", "app").await.unwrap();
    assert_eq!(spaces[0].role, SpaceRole::Editor);
    assert_eq!(spaces[0].space.name, "Home");
    assert_eq!(
        c.get_space("org", "app", "sp1").await.unwrap().role,
        SpaceRole::Editor
    );
    c.rename_space("org", "app", "sp1", "Renamed")
        .await
        .unwrap();
    c.delete_space("org", "app", "sp1").await.unwrap();
}

#[tokio::test]
async fn invalid_arguments_fail_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    let unknown = SpaceCreate {
        name: "x".into(),
        kind: SpaceKind::Other,
    };
    assert!(matches!(
        c.create_space("org", "app", &unknown).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.get_space("org", "app", "a/b").await,
        Err(CopepodError::InvalidArgument(_))
    ));
    let moderator = SpaceInviteCreate {
        email: "a@x.com".into(),
        role: SpaceRole::Other,
    };
    assert!(matches!(
        c.create_space_invite("org", "app", "sp1", &moderator).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.accept_space_invite("org", "app", " ").await,
        Err(CopepodError::InvalidArgument(_))
    ));
    for options in [
        RecordUpdateOptions {
            space_id: Some("a/b".into()),
            if_match: None,
        },
        RecordUpdateOptions {
            space_id: None,
            if_match: Some("".into()),
        },
        RecordUpdateOptions {
            space_id: None,
            if_match: Some("  ".into()),
        },
    ] {
        assert!(matches!(
            c.update_record_with("org", "app", "notes", "r1", &json!({}), &options)
                .await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    assert!(matches!(
        c.records("org", "app", "notes").space("a b").list().await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.records("org", "app", "notes")
            .space("")
            .get_one("r1")
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn members_and_role_errors() {
    let server = MockServer::start().await;
    let member = json!({"space_id": "sp1", "user_id": "u2", "role": "viewer",
        "added_by": "u1", "created": "2026-10-01T00:00:00Z"});
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/sp1/members")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [member.clone()]})))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{APP}/spaces/sp1/members/u2")))
        .and(body_json(json!({"role": "editor"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(member))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{APP}/spaces/sp1/members/u1")))
        .respond_with(ResponseTemplate::new(409).set_body_json(
            json!({"status": 409, "code": "last_owner", "message": "a Space keeps one owner"}),
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/nope/members")))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"code": "space_not_found", "message": "space not found"})),
        )
        .mount(&server)
        .await;

    let c = client(&server);
    let members = c.list_space_members("org", "app", "sp1").await.unwrap();
    assert_eq!(members[0].role, SpaceRole::Viewer);
    let updated = c
        .set_space_member_role("org", "app", "sp1", "u2", SpaceRole::Editor)
        .await
        .unwrap();
    assert_eq!(updated.user_id, "u2");
    let err = c
        .remove_space_member("org", "app", "sp1", "u1")
        .await
        .unwrap_err();
    assert_eq!(
        (err.api_status(), err.api_code()),
        (Some(409), Some("last_owner"))
    );
    let err = c
        .list_space_members("org", "app", "nope")
        .await
        .unwrap_err();
    assert!(err.is_space_not_found());
}

#[tokio::test]
async fn invites_create_list_revoke_accept() {
    let server = MockServer::start().await;
    let invite = json!({"id": "inv1", "space_id": "sp1", "email": "a@x.com", "role": "editor",
        "invited_by": "u1", "status": "pending", "expires_at": "2026-10-31T00:00:00Z",
        "created": "2026-10-17T00:00:00Z"});
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/sp1/invites")))
        .and(body_json(json!({"email": "a@x.com", "role": "editor"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(invite.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/spaces/sp1/invites")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [invite]})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{APP}/spaces/sp1/invites/inv1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"revoked": true})))
        .expect(1)
        .mount(&server)
        .await;
    let mut accepted = space("sp1");
    accepted["role"] = json!("editor");
    Mock::given(method("POST"))
        .and(path(format!("{APP}/space-invites/accept")))
        .and(body_json(json!({"token": "tok"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(accepted))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/space-invites/accept")))
        .and(body_json(json!({"token": "old"})))
        .respond_with(
            ResponseTemplate::new(410)
                .set_body_json(json!({"code": "invite_expired", "message": "invite expired"})),
        )
        .mount(&server)
        .await;

    let c = client(&server);
    let created = c
        .create_space_invite(
            "org",
            "app",
            "sp1",
            &SpaceInviteCreate {
                email: "a@x.com".into(),
                role: SpaceRole::Editor,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        (created.status.as_str(), created.role),
        ("pending", SpaceRole::Editor)
    );
    assert_eq!(
        c.list_space_invites("org", "app", "sp1")
            .await
            .unwrap()
            .len(),
        1
    );
    c.revoke_space_invite("org", "app", "sp1", "inv1")
        .await
        .unwrap();
    let joined = c.accept_space_invite("org", "app", "tok").await.unwrap();
    assert_eq!(
        (joined.space.id.as_str(), joined.role),
        ("sp1", SpaceRole::Editor)
    );
    let err = c
        .accept_space_invite("org", "app", "old")
        .await
        .unwrap_err();
    assert_eq!(
        (err.api_status(), err.api_code()),
        (Some(410), Some("invite_expired"))
    );
}

#[tokio::test]
async fn space_records_send_x_space_id_and_if_match() {
    let server = MockServer::start().await;
    let records = format!("{APP}/records/notes");
    Mock::given(method("GET"))
        .and(path(records.clone()))
        .and(header("x-space-id", "sp1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1, "per_page": 20, "total_items": 0, "total_pages": 0, "items": []})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(records.clone()))
        .and(header("x-space-id", "sp1"))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "r1"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{records}/r1")))
        .and(header("x-space-id", "sp1"))
        .and(header("if-match", "3"))
        .and(body_json(json!({"title": "t"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "r1", "rev": 4})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{records}/r1")))
        .and(header("if-match", "2"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "status": 409, "code": "revision_conflict", "current_rev": 4,
            "message": "the record changed since the revision you hold"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{records}/r1")))
        .and(header("x-space-id", "sp1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let c = client(&server);
    c.records("org", "app", "notes")
        .space("sp1")
        .list()
        .await
        .unwrap();
    c.create_record_in_space("org", "app", "notes", "sp1", &json!({"title": "t"}))
        .await
        .unwrap();
    let opts = |rev: &str| RecordUpdateOptions {
        space_id: Some("sp1".into()),
        if_match: Some(rev.into()),
    };
    let ok = c
        .update_record_with(
            "org",
            "app",
            "notes",
            "r1",
            &json!({"title": "t"}),
            &opts("3"),
        )
        .await
        .unwrap();
    assert_eq!(ok["rev"], 4);
    let err = c
        .update_record_with(
            "org",
            "app",
            "notes",
            "r1",
            &json!({"title": "t"}),
            &opts("2"),
        )
        .await
        .unwrap_err();
    assert!(err.is_revision_conflict());
    assert_eq!(err.api_status(), Some(409));
    assert_eq!(
        err.revision_conflict_details().unwrap().current_rev,
        json!(4)
    );
    c.delete_record_in_space("org", "app", "notes", "sp1", "r1")
        .await
        .unwrap();
}

#[tokio::test]
async fn plain_update_sends_no_space_or_if_match_header() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path(format!("{APP}/records/notes/r1")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "r1"})))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .update_record_with("org", "app", "notes", "r1", &json!({}), &Default::default())
        .await
        .unwrap();
    let req = &server.received_requests().await.unwrap()[0];
    assert!(req.headers.get("x-space-id").is_none());
    assert!(req.headers.get("if-match").is_none());
}

#[tokio::test]
async fn space_required_and_role_errors_keep_their_codes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/records/notes")))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"code": "space_required", "message": "send X-Space-Id"})),
        )
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{APP}/records/notes/r1")))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(
                json!({"code": "insufficient_role", "message": "role cannot do this"}),
            ),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let err = c.records("org", "app", "notes").list().await.unwrap_err();
    assert_eq!(err.api_code(), Some("space_required"));
    let err = c
        .delete_record_in_space("org", "app", "notes", "sp1", "r1")
        .await
        .unwrap_err();
    assert!(err.is_insufficient_role());
}

#[tokio::test]
async fn space_files_send_x_space_id() {
    let server = MockServer::start().await;
    let file = format!("{APP}/files/notes/r1/a.txt");
    Mock::given(method("POST"))
        .and(path(file.clone()))
        .and(header("x-space-id", "sp1"))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("content-type", "text/plain"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"filename": "a.txt"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(file.clone()))
        .and(header("x-space-id", "sp1"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello".to_vec()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(file))
        .and(header("x-space-id", "sp1"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_json(json!({"code": "insufficient_role", "message": "viewer"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    c.upload_file_in_space(
        "org",
        "app",
        "notes",
        "sp1",
        "r1",
        b"hello".to_vec(),
        "a.txt",
        "text/plain",
    )
    .await
    .unwrap();
    let bytes = c
        .download_file_in_space("org", "app", "notes", "sp1", "r1", "a.txt")
        .await
        .unwrap();
    assert_eq!(&bytes[..], b"hello");
    let err = c
        .delete_file_in_space("org", "app", "notes", "sp1", "r1", "a.txt")
        .await
        .unwrap_err();
    assert!(err.is_insufficient_role());
}

#[tokio::test]
async fn space_realtime_passes_space_id_and_decodes_it() {
    let server = MockServer::start().await;
    let body = "event: ready\ndata: {}\n\n\
        event: record\nid: 7\ndata: {\"id\":7,\"action\":\"create\",\"collection\":\"notes\",\"record\":{\"id\":\"r1\"},\"space_id\":\"sp1\"}\n\n\
        event: record\nid: 8\ndata: {\"id\":8,\"action\":\"create\",\"collection\":\"other\",\"record\":{}}\n\n";
    Mock::given(method("GET"))
        .and(path(format!("{APP}/realtime")))
        .and(query_param("space_id", "sp1"))
        .and(header_exists("authorization"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/realtime")))
        .and(query_param("space_id", "gone"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"code": "space_not_found", "message": "space not found"})),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let mut stream = c
        .subscribe_space("org", "app", "sp1", RealtimeSubscriptionOptions::default())
        .await
        .unwrap();
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first.space_id.as_deref(), Some("sp1"));
    let second = stream.next().await.unwrap().unwrap();
    assert_eq!(second.space_id, None);
    let err = c
        .subscribe_space("org", "app", "gone", RealtimeSubscriptionOptions::default())
        .await
        .err()
        .unwrap();
    assert!(err.is_space_not_found());
}

#[tokio::test]
async fn space_get_one_sends_x_space_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/records/notes/r1")))
        .and(header("x-space-id", "sp1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "r1"})))
        .expect(1)
        .mount(&server)
        .await;
    let rec = client(&server)
        .records("org", "app", "notes")
        .space("sp1")
        .get_one("r1")
        .await
        .unwrap();
    assert_eq!(rec["id"], "r1");
}

#[tokio::test]
async fn plain_subscribe_omits_space_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/realtime")))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string("event: ready\ndata: {}\n\n"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let _stream = client(&server).subscribe("org", "app").await.unwrap();
    let req = &server.received_requests().await.unwrap()[0];
    assert!(req.url.query_pairs().all(|(k, _)| k != "space_id"));
    assert!(req.headers.get("x-api-key").is_none());
}

#[tokio::test]
async fn space_calls_never_send_the_api_key() {
    let server = MockServer::start().await;
    let ok = ResponseTemplate::new(200).set_body_json(json!({"items": [], "page": 1,
        "per_page": 20, "total_items": 0, "total_pages": 0, "id": "r1"}));
    Mock::given(wiremock::matchers::any())
        .respond_with(ok)
        .mount(&server)
        .await;
    let c = client(&server);
    c.list_spaces("org", "app").await.unwrap();
    c.list_space_members("org", "app", "sp1").await.unwrap();
    c.records("org", "app", "notes")
        .space("sp1")
        .list()
        .await
        .unwrap();
    c.create_record_in_space("org", "app", "notes", "sp1", &json!({}))
        .await
        .unwrap();
    c.update_record_with(
        "org",
        "app",
        "notes",
        "r1",
        &json!({}),
        &RecordUpdateOptions {
            space_id: Some("sp1".into()),
            if_match: Some("1".into()),
        },
    )
    .await
    .unwrap();
    c.upload_file_in_space(
        "org",
        "app",
        "notes",
        "sp1",
        "r1",
        vec![1],
        "a",
        "text/plain",
    )
    .await
    .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 6);
    for req in requests {
        assert!(req.headers.get("x-api-key").is_none(), "{}", req.url);
        assert!(req.headers.get("authorization").is_some(), "{}", req.url);
    }
}

#[tokio::test]
async fn commons_can_be_created_and_contributors_invited() {
    let server = MockServer::start().await;
    let mut commons = space("c1");
    commons["kind"] = json!("commons");
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces")))
        .and(body_json(json!({"name": "Wiki", "kind": "commons"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(commons))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces/c1/invites")))
        .and(body_json(
            json!({"email": "a@x.com", "role": "contributor"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "i1", "space_id": "c1", "email": "a@x.com", "role": "contributor",
            "status": "pending", "expires_at": "2026-11-01T00:00:00Z"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{APP}/spaces")))
        .and(body_json(json!({"name": "Again", "kind": "commons"})))
        .respond_with(ResponseTemplate::new(429).set_body_json(
            json!({"code": "space_rate_limited", "message": "too many spaces created today"}),
        ))
        .mount(&server)
        .await;
    let c = client(&server);
    let made = c
        .create_space(
            "org",
            "app",
            &SpaceCreate {
                name: "Wiki".into(),
                kind: SpaceKind::Commons,
            },
        )
        .await
        .unwrap();
    assert_eq!(made.kind, SpaceKind::Commons);
    let invite = c
        .create_space_invite(
            "org",
            "app",
            "c1",
            &SpaceInviteCreate {
                email: "a@x.com".into(),
                role: SpaceRole::Contributor,
            },
        )
        .await
        .unwrap();
    assert_eq!(invite.role, SpaceRole::Contributor);
    let e = c
        .create_space(
            "org",
            "app",
            &SpaceCreate {
                name: "Again".into(),
                kind: SpaceKind::Commons,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(429), Some("space_rate_limited"))
    );
}
