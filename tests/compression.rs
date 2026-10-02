use std::io::Write;

use copepod_sdk::CopepodClient;
use flate2::{write::GzEncoder, Compression};
use serde_json::json;
use wiremock::matchers::{header_regex, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const LIST_PATH: &str = "/api/platform/orgs/o/apps/a/records/c";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("tok")
        .auto_refresh(false)
        .build()
        .unwrap()
}

#[tokio::test]
async fn list_sends_accept_encoding_gzip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(LIST_PATH))
        .and(header_regex("accept-encoding", "gzip"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "page": 1, "per_page": 500, "total_items": 0, "total_pages": 0, "items": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server)
        .app("o", "a")
        .records("c")
        .query()
        .list()
        .await
        .unwrap();
}

#[tokio::test]
async fn gzip_encoded_list_body_is_decoded() {
    let body = json!({
        "page": 1, "per_page": 500, "total_items": 2, "total_pages": 1,
        "items": [{"id": "r1", "title": "one"}, {"id": "r2", "title": "two"}]
    });
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(body.to_string().as_bytes()).unwrap();
    let gz = enc.finish().unwrap();

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(LIST_PATH))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .insert_header("content-encoding", "gzip")
                .set_body_bytes(gz),
        )
        .mount(&server)
        .await;

    let list = client(&server)
        .app("o", "a")
        .records("c")
        .query()
        .list()
        .await
        .unwrap();
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[1]["title"], "two");
}
