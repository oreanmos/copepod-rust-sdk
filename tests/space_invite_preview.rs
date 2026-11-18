use copepod_sdk::error::*;
use copepod_sdk::{
    CopepodClient, CopepodError, SpaceInvitePreview, SpaceInvitePreviewStatus, SpaceRole,
};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PREVIEW: &str = "/api/platform/orgs/org/apps/app/space-invites/preview";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("must-not-send")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn coded(status: u16, code: &str) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({"code": code, "message": "m"}))
}

async fn mount(server: &MockServer, resp: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path(PREVIEW))
        .respond_with(resp)
        .mount(server)
        .await;
}

fn null_card(status: &str) -> serde_json::Value {
    json!({"space_name": null, "role": null, "inviter_email": null, "member_count": null,
        "invited_email_masked": null, "created": null,
        "expires_at": "2026-11-20T00:00:00Z", "status": status})
}

async fn preview(c: &CopepodClient) -> Result<SpaceInvitePreview> {
    c.preview_space_invite("org", "app", "tok-abc").await
}

#[tokio::test]
async fn pending_card_is_typed() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "space_name": "Home", "role": "editor", "inviter_email": "a@example.com",
            "member_count": 3, "invited_email_masked": "r***@example.com",
            "created": "2026-11-10T00:00:00Z", "expires_at": "2026-11-20T00:00:00Z",
            "status": "pending"})),
    )
    .await;
    let p = preview(&client(&server)).await.unwrap();
    assert_eq!(p.status, SpaceInvitePreviewStatus::Pending);
    assert_eq!(p.space_name.as_deref(), Some("Home"));
    assert_eq!(p.role, Some(SpaceRole::Editor));
    assert_eq!(p.inviter_email.as_deref(), Some("a@example.com"));
    assert_eq!(p.member_count, Some(3));
    assert_eq!(p.invited_email_masked.as_deref(), Some("r***@example.com"));
    assert_eq!(p.created.as_deref(), Some("2026-11-10T00:00:00Z"));
    assert_eq!(p.expires_at, "2026-11-20T00:00:00Z");
}

#[tokio::test]
async fn non_pending_states_have_null_card_fields() {
    for (raw, want) in [
        ("expired", SpaceInvitePreviewStatus::Expired),
        ("revoked", SpaceInvitePreviewStatus::Revoked),
        ("accepted", SpaceInvitePreviewStatus::Accepted),
    ] {
        let server = MockServer::start().await;
        mount(
            &server,
            ResponseTemplate::new(200).set_body_json(null_card(raw)),
        )
        .await;
        let p = preview(&client(&server)).await.unwrap();
        assert_eq!(p.status, want, "{raw}");
        assert!(p.space_name.is_none() && p.role.is_none() && p.inviter_email.is_none());
        assert!(p.member_count.is_none() && p.invited_email_masked.is_none());
        assert!(p.created.is_none());
        assert_eq!(p.expires_at, "2026-11-20T00:00:00Z");
    }
}

#[tokio::test]
async fn unknown_status_and_role_fall_back() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "space_name": "Home", "role": "supervisor", "inviter_email": null,
            "member_count": 1, "invited_email_masked": null, "created": null,
            "expires_at": "2026-11-20T00:00:00Z", "status": "paused"})),
    )
    .await;
    let p = preview(&client(&server)).await.unwrap();
    assert_eq!(p.status, SpaceInvitePreviewStatus::Other);
    assert_eq!(p.role, Some(SpaceRole::Other));
}

#[tokio::test]
async fn unknown_token_is_404_invite_not_found() {
    let server = MockServer::start().await;
    mount(&server, coded(404, "invite_not_found")).await;
    let e: CopepodError = preview(&client(&server)).await.unwrap_err();
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(404), Some(INVITE_NOT_FOUND_CODE))
    );
}

#[tokio::test]
async fn throttled_is_429_link_rate_limited() {
    let server = MockServer::start().await;
    mount(&server, coded(429, "link_rate_limited")).await;
    let e = preview(&client(&server)).await.unwrap_err();
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(429), Some(LINK_RATE_LIMITED_CODE))
    );
}

#[tokio::test]
async fn throttle_unavailable_is_503() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(503).set_body_json(json!({"message": "m"})),
    )
    .await;
    let e = preview(&client(&server)).await.unwrap_err();
    assert_eq!(e.api_status(), Some(503));
}

#[tokio::test]
async fn request_is_token_only_and_sends_no_credentials() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(null_card("expired")),
    )
    .await;
    preview(&client(&server)).await.unwrap();
    let reqs = server.received_requests().await.unwrap();
    assert_eq!(reqs.len(), 1);
    let r = &reqs[0];
    assert_eq!(r.url.path(), PREVIEW);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&r.body).unwrap(),
        json!({"token": "tok-abc"})
    );
    assert!(!r.headers.contains_key("authorization"));
    assert!(!r.headers.contains_key("x-api-key"));
}

#[tokio::test]
async fn blank_token_is_refused_before_any_request() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(null_card("expired")),
    )
    .await;
    let e = client(&server)
        .preview_space_invite("org", "app", "  ")
        .await
        .unwrap_err();
    assert!(matches!(e, CopepodError::InvalidArgument(_)), "{e:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}
