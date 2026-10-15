use copepod_sdk::CopepodClient;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The server answers `{"items": [...]}` with no pagination envelope.
#[tokio::test]
async fn list_flags_decodes_the_items_only_server_shape() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/platform/apps/app_1/flags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [{
                "id": "flag_1",
                "app_id": "app_1",
                "key": "beta",
                "enabled": true,
                "description": null,
                "rules": null,
                "created": "2026-10-01T00:00:00Z",
                "updated": "2026-10-01T00:00:00Z"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap();

    let flags = client.list_flags("app_1").await.unwrap();
    assert_eq!(flags.items.len(), 1);
    assert_eq!(flags.items[0].key, "beta");
    assert!(flags.items[0].enabled);
}
