use copepod_sdk::{CopepodClient, ImportExecuteRequest};
use serde_json::json;
use wiremock::matchers::{body_bytes, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const IMPORT_ID: &str = "0123456789abcdef0123456789abcdef";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("t")
        .auto_refresh(false)
        .build()
        .unwrap()
}

#[tokio::test]
async fn discover_uploads_raw_bytes_and_returns_import_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o/apps/a/import/discover"))
        .and(body_bytes(b"SQLite format 3\0".to_vec()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "import_id": IMPORT_ID,
            "tables": [{
                "name": "notes",
                "row_count": 3,
                "columns": [{
                    "name": "id", "sqlite_type": "INTEGER", "copepod_type": "integer",
                    "not_null": true, "is_pk": true
                }]
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let resp = client(&server)
        .import_discover("o", "a", b"SQLite format 3\0".to_vec())
        .await
        .unwrap();
    assert_eq!(resp.import_id, IMPORT_ID);
    assert_eq!(resp.tables[0].name, "notes");
    assert_eq!(resp.tables[0].row_count, 3);
    assert!(resp.tables[0].columns[0].is_pk);
}

#[tokio::test]
async fn execute_sends_only_import_id_and_tables() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/platform/orgs/o/apps/a/import/execute"))
        .and(body_json(
            json!({"import_id": IMPORT_ID, "tables": ["notes"]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tables_imported": 1, "rows_imported": 3, "errors": []
        })))
        .expect(1)
        .mount(&server)
        .await;
    let req = ImportExecuteRequest {
        import_id: IMPORT_ID.into(),
        tables: vec!["notes".into()],
    };
    let resp = client(&server)
        .import_execute("o", "a", &req)
        .await
        .unwrap();
    assert_eq!(resp.tables_imported, 1);
    assert_eq!(resp.rows_imported, 3);
    assert!(resp.errors.is_empty());
}
