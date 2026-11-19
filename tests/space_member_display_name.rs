//! toolkit-ui4-ui5 S9: `SpaceMember.display_name` (null for hidden or unset
//! names; absent from an old server). Red by compile error: the field does
//! not exist yet and a stub cannot add it without making these pass.

use copepod_sdk::CopepodClient;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .auto_refresh(false)
        .build()
        .unwrap()
}

async fn members(body: serde_json::Value) -> Vec<copepod_sdk::SpaceMember> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/platform/orgs/org/apps/app/spaces/sp1/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    client(&server)
        .list_space_members("org", "app", "sp1")
        .await
        .unwrap()
}

fn member(user: &str) -> serde_json::Value {
    json!({"space_id": "sp1", "user_id": user, "role": "editor", "added_by": "u0",
        "created": "2026-11-01T00:00:00Z"})
}

#[tokio::test]
async fn display_name_string_and_null_parse() {
    let mut named = member("u1");
    named["display_name"] = json!("Ana");
    let mut unset = member("u2");
    unset["display_name"] = json!(null);
    let list = members(json!({"items": [named, unset]})).await;
    assert_eq!(list[0].display_name.as_deref(), Some("Ana"));
    assert_eq!(list[1].display_name, None);
}

#[tokio::test]
async fn old_server_member_without_display_name_still_parses() {
    let list = members(json!({"items": [member("u1")]})).await;
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].display_name, None);
}
