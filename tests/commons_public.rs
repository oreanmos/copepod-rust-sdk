use copepod_sdk::error::*;
use copepod_sdk::{CommonsRecordsQuery, CopepodClient, CopepodError, SpaceKind};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
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

async fn err_of<T: std::fmt::Debug>(r: Result<T>) -> CopepodError {
    r.expect_err("expected an error")
}

fn coded(status: u16, code: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"}))
}

struct NoCredentials(serde_json::Value);
impl Respond for NoCredentials {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        if req.headers.contains_key("authorization") || req.headers.contains_key("x-api-key") {
            coded(400, "leaked")
        } else {
            ResponseTemplate::new(200).set_body_json(self.0.clone())
        }
    }
}

#[tokio::test]
async fn public_reads_send_no_credentials_and_the_query() {
    let server = MockServer::start().await;
    let base = format!("{APP}/commons/c1");
    Mock::given(method("GET"))
        .and(path(base.clone()))
        .respond_with(NoCredentials(
            json!({"id": "c1", "name": "Recipes", "kind": "commons"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/records/notes")))
        .and(query_param("q", "50%_off"))
        .and(query_param("filter", "title='a~b'"))
        .and(query_param("sort", "-created"))
        .and(query_param("fields", "id,title"))
        .and(query_param("page", "2"))
        .and(query_param("per_page", "10"))
        .respond_with(NoCredentials(json!({
            "page": 2, "per_page": 10, "total_items": 1, "total_pages": 1,
            "items": [{"id": "r1", "title": "t"}]})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/records/notes/r1")))
        .respond_with(NoCredentials(json!({"id": "r1"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/records/notes/r1/files/a.png")))
        .respond_with(NoCredentials(
            json!({"url": "/api/platform/apps/app/files/signed/k?token=t&expires=1760000000&anon=1", "expires": 1760000000}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    let commons = c.get_public_commons("org", "app", "c1").await.unwrap();
    assert_eq!(commons.name, "Recipes");
    assert_eq!(commons.kind, SpaceKind::Commons);
    let page = c
        .list_public_commons_records(
            "org",
            "app",
            "c1",
            "notes",
            &CommonsRecordsQuery {
                q: Some("50%_off".into()),
                filter: Some("title='a~b'".into()),
                sort: Some("-created".into()),
                fields: Some("id,title".into()),
                page: Some(2),
                per_page: Some(10),
            },
        )
        .await
        .unwrap();
    assert_eq!((page.page, page.items.len()), (2, 1));
    let one = c
        .get_public_commons_record("org", "app", "c1", "notes", "r1")
        .await
        .unwrap();
    assert_eq!(one["id"], "r1");
    let file = c
        .sign_public_commons_file("org", "app", "c1", "notes", "r1", "a.png")
        .await
        .unwrap();
    assert_eq!(file.expires, 1760000000);
    assert!(file.url.starts_with("/api/platform/apps/app/files/signed/"));
    assert!(file.url.contains("anon=1"));
    for req in server.received_requests().await.unwrap() {
        assert!(!req.headers.contains_key("authorization"));
        assert!(!req.headers.contains_key("x-api-key"));
    }
}

#[tokio::test]
async fn blank_q_is_not_sent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{APP}/commons/c1/records/notes")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1, "per_page": 30, "total_items": 0, "total_pages": 0, "items": []})))
        .mount(&server)
        .await;
    let q = CommonsRecordsQuery {
        q: Some("  ".into()),
        ..Default::default()
    };
    client(&server)
        .list_public_commons_records("org", "app", "c1", "notes", &q)
        .await
        .unwrap();
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs[0].url.query(), None);
}

#[tokio::test]
async fn public_read_errors_keep_their_codes() {
    let server = MockServer::start().await;
    let base = format!("{APP}/commons");
    Mock::given(method("GET"))
        .and(path(format!("{base}/closed")))
        .respond_with(coded(404, "space_not_found"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/spam")))
        .respond_with(coded(429, "link_rate_limited"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/closed/records/secret")))
        .respond_with(coded(404, "space_not_found"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/c1/records/hidden")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "not found"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/closed/records/notes/r1")))
        .respond_with(coded(404, "space_not_found"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/closed/records/notes/r1/files/a.png")))
        .respond_with(coded(404, "space_not_found"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{base}/c1/records/notes")))
        .and(query_param("filter", "a=1"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(
                json!({"message": "filter names a field that is not available here"}),
            ),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let e = err_of(c.get_public_commons("org", "app", "closed").await).await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(404), Some(SPACE_NOT_FOUND_CODE))
    );
    let e = err_of(c.get_public_commons("org", "app", "spam").await).await;
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(429), Some(LINK_RATE_LIMITED_CODE))
    );
    let q = CommonsRecordsQuery::default();
    let e = err_of(
        c.list_public_commons_records("org", "app", "closed", "secret", &q)
            .await,
    )
    .await;
    assert_eq!(e.api_code(), Some("space_not_found"));
    let e = err_of(
        c.list_public_commons_records("org", "app", "c1", "hidden", &q)
            .await,
    )
    .await;
    assert_eq!((e.api_status(), e.api_code()), (Some(404), None));
    let e = err_of(
        c.list_public_commons_records(
            "org",
            "app",
            "c1",
            "notes",
            &CommonsRecordsQuery {
                filter: Some("a=1".into()),
                ..Default::default()
            },
        )
        .await,
    )
    .await;
    assert_eq!(e.api_status(), Some(400));
    let e = err_of(
        c.get_public_commons_record("org", "app", "closed", "notes", "r1")
            .await,
    )
    .await;
    assert_eq!(e.api_code(), Some("space_not_found"));
    let e = err_of(
        c.sign_public_commons_file("org", "app", "closed", "notes", "r1", "a.png")
            .await,
    )
    .await;
    assert_eq!(e.api_code(), Some("space_not_found"));
}

#[tokio::test]
async fn set_space_anonymous_read_patches_with_the_bearer() {
    let server = MockServer::start().await;
    let space = |on: bool| {
        json!({"id": "c1", "app_id": "app", "name": "R", "kind": "commons",
               "owner_user_id": "u", "anonymous_read": on, "created": "2026-10-20T00:00:00Z"})
    };
    Mock::given(method("PATCH"))
        .and(path(format!("{APP}/spaces/c1")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(json!({"anonymous_read": true})))
        .respond_with(ResponseTemplate::new(200).set_body_json(space(true)))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("{APP}/spaces/c1")))
        .and(body_json(json!({"anonymous_read": false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(space(false)))
        .expect(1)
        .mount(&server)
        .await;
    for (id, status, code) in [
        ("nope", 404, "space_not_found"),
        ("private", 400, "not_commons"),
        ("member", 403, "insufficient_role"),
        ("switchoff", 409, "anonymous_commons_disabled"),
        ("rolling", 503, "anonymous_commons_upgrading"),
    ] {
        Mock::given(method("PATCH"))
            .and(path(format!("{APP}/spaces/{id}")))
            .respond_with(coded(status, code))
            .mount(&server)
            .await;
    }
    let c = client(&server);
    assert!(
        c.set_space_anonymous_read("org", "app", "c1", true)
            .await
            .unwrap()
            .anonymous_read
    );
    assert!(
        !c.set_space_anonymous_read("org", "app", "c1", false)
            .await
            .unwrap()
            .anonymous_read
    );
    for (id, status, code) in [
        ("nope", 404, SPACE_NOT_FOUND_CODE),
        ("private", 400, NOT_COMMONS_CODE),
        ("member", 403, INSUFFICIENT_ROLE_CODE),
        ("switchoff", 409, ANONYMOUS_COMMONS_DISABLED_CODE),
        ("rolling", 503, ANONYMOUS_COMMONS_UPGRADING_CODE),
    ] {
        let e = err_of(c.set_space_anonymous_read("org", "app", id, true).await).await;
        assert_eq!((e.api_status(), e.api_code()), (Some(status), Some(code)));
    }
}

fn app(settings: serde_json::Value) -> serde_json::Value {
    json!({"id": "app", "name": "A", "slug": "a", "org_id": "org", "settings": settings,
           "created": "2026-10-20T00:00:00Z", "updated": "2026-10-20T00:00:00Z"})
}

#[tokio::test]
async fn master_switch_patches_settings_and_reads_back() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/platform/orgs/org/apps/app"))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(
            json!({"settings": {"anonymous_commons_enabled": false}}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(app(json!({"anonymous_commons_enabled": false, "k": 1}))),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/platform/orgs/org/apps/app"))
        .and(body_json(
            json!({"settings": {"anonymous_commons_enabled": null}}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(app(json!({}))))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/api/platform/orgs/org/apps/other"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({"message": "no"})))
        .mount(&server)
        .await;
    let c = client(&server);
    let off = c
        .set_anonymous_commons_enabled("org", "app", Some(false))
        .await
        .unwrap();
    assert!(!off.anonymous_commons_enabled());
    let cleared = c
        .set_anonymous_commons_enabled("org", "app", None)
        .await
        .unwrap();
    assert!(cleared.anonymous_commons_enabled());
    let e = err_of(
        c.set_anonymous_commons_enabled("org", "other", Some(true))
            .await,
    )
    .await;
    assert_eq!(e.api_status(), Some(403));
}

#[tokio::test]
async fn bad_arguments_fail_before_the_network() {
    let server = MockServer::start().await;
    let c = client(&server);
    let q = CommonsRecordsQuery::default();
    assert!(matches!(
        c.get_public_commons("org", "app", "a/b").await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.list_public_commons_records("org", "app", "c1", "", &q)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.sign_public_commons_file("org", "app", "c1", "n", "r", "a/b.png")
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    assert!(matches!(
        c.set_space_anonymous_read("org", "app", "", true).await,
        Err(CopepodError::InvalidArgument(_))
    ));
    // The server trims q before it counts, so padding does not count.
    for q in [format!(" {} ", "a".repeat(100)), " ".repeat(200)] {
        let padded = CommonsRecordsQuery {
            q: Some(q),
            ..Default::default()
        };
        assert!(!matches!(
            c.list_public_commons_records("org", "app", "c1", "n", &padded)
                .await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    for id in [".", ".."] {
        assert!(matches!(
            c.get_public_commons("org", "app", id).await,
            Err(CopepodError::InvalidArgument(_))
        ));
        assert!(matches!(
            c.delete_space("org", "app", id).await,
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    let long = CommonsRecordsQuery {
        q: Some("x".repeat(101)),
        ..Default::default()
    };
    assert!(matches!(
        c.list_public_commons_records("org", "app", "c1", "n", &long)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    let ok_long = CommonsRecordsQuery {
        q: Some("é".repeat(100)),
        ..Default::default()
    };
    // 100 multi-byte characters pass the length check (the mock-less call
    // then fails on the network, not on validation).
    assert!(!matches!(
        c.list_public_commons_records("org", "app", "c1", "n", &ok_long)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
    for bad in [
        "title ~ 'a'",
        "title !~ 'a'",
        "a=1 && b=2 && c=3 && d=4 && e=5 && f=6 && g=7 && h=8 && i=9",
    ] {
        let query = CommonsRecordsQuery {
            filter: Some(bad.into()),
            ..Default::default()
        };
        assert!(
            matches!(
                c.list_public_commons_records("org", "app", "c1", "n", &query)
                    .await,
                Err(CopepodError::InvalidArgument(_))
            ),
            "{bad}"
        );
    }
    // Eight comparisons and quoted operators are allowed through.
    let ok = CommonsRecordsQuery {
        filter: Some("a=1 && b!='x~y' && c>=2 && d<3 && e='=' && f=1 && g=1 && h=1".into()),
        ..Default::default()
    };
    assert!(!matches!(
        c.list_public_commons_records("org", "app", "c1", "n", &ok)
            .await,
        Err(CopepodError::InvalidArgument(_))
    ));
}

#[test]
fn error_code_constants_match_the_server() {
    assert_eq!(
        ANONYMOUS_COMMONS_DISABLED_CODE,
        "anonymous_commons_disabled"
    );
    assert_eq!(
        ANONYMOUS_COMMONS_UPGRADING_CODE,
        "anonymous_commons_upgrading"
    );
}

#[tokio::test]
async fn download_follows_the_relative_url_without_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/platform/apps/app/files/signed/k"))
        .and(query_param("anon", "1"))
        .respond_with(|req: &Request| {
            if req.headers.contains_key("authorization") || req.headers.contains_key("x-api-key") {
                coded(400, "leaked")
            } else {
                ResponseTemplate::new(200).set_body_bytes(b"bytes".to_vec())
            }
        })
        .expect(1)
        .mount(&server)
        .await;
    for (key, status, code) in [("old", 400, "expired"), ("off", 401, "unauthorized")] {
        Mock::given(method("GET"))
            .and(path(format!("/api/platform/apps/app/files/signed/{key}")))
            .respond_with(coded(status, code))
            .mount(&server)
            .await;
    }
    let c = client(&server);
    let file = |p: &str| {
        serde_json::from_value::<copepod_sdk::PublicFileUrl>(json!({"url": p, "expires": 1}))
            .unwrap()
    };
    let got = c
        .download_public_commons_file(&file(
            "/api/platform/apps/app/files/signed/k?token=t&expires=9&anon=1",
        ))
        .await
        .unwrap();
    assert_eq!(&got[..], b"bytes");
    let e = err_of(
        c.download_public_commons_file(&file("/api/platform/apps/app/files/signed/old?token=t"))
            .await,
    )
    .await;
    assert_eq!(e.api_status(), Some(400));
    let e = err_of(
        c.download_public_commons_file(&file("/api/platform/apps/app/files/signed/off?token=t"))
            .await,
    )
    .await;
    assert_eq!(e.api_status(), Some(401));
    for bad in ["https://evil.example/x", "//evil.example/x", "rel/path"] {
        assert!(matches!(
            c.resolve_url(bad),
            Err(CopepodError::InvalidArgument(_))
        ));
    }
    assert!(c
        .resolve_url("/api/x?y=1")
        .unwrap()
        .as_str()
        .ends_with("/api/x?y=1"));
}
