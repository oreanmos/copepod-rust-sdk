use copepod_sdk::{CopepodClient, CopepodError, DeployOptions, ReleasePhase};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BASE: &str = "/api/platform/orgs/org_1/deployments/dep_1";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .auto_refresh(false)
        .token("platform-token")
        .build()
        .unwrap()
}

fn full_release() -> serde_json::Value {
    json!({
        "phase": "rolling_out",
        "version": 4519022873219374u64,
        "configured_image": "registry.example/team/app:latest",
        "runtime_available": true,
        "serving": {"image": "registry.example/team/app@sha256:aa", "log_id": "log_0",
                    "deployed_at": "2026-09-29T10:00:00Z", "ready": 1, "desired": 1},
        "target": {"log_id": "log_1", "action": "deploy",
                   "requested_image": "registry.example/team/app:latest",
                   "resolved_image": "registry.example/team/app@sha256:bb",
                   "plan": "surge", "triggered_by": "user_1"},
        "progress": {"updated": 1, "updated_ready": 0, "desired": 1, "previous_serving": 1,
                     "current_step": "Wait for rollout",
                     "steps": [{"name": "Wait for rollout", "status": "running", "message": "",
                                "started": "2026-09-29T10:05:00Z", "finished": null}]},
        "started_at": "2026-09-29T10:05:00Z",
        "finished_at": null,
        "outcome": null
    })
}

async fn mock_release(server: &MockServer, body: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path(format!("{BASE}/release")))
        .and(header("authorization", "Bearer platform-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn release_full_payload_deserialises() {
    let server = MockServer::start().await;
    mock_release(&server, full_release()).await;
    let release = client(&server)
        .get_deployment_release("org_1", "dep_1")
        .await
        .unwrap();
    assert_eq!(release.phase, ReleasePhase::RollingOut);
    assert_eq!(release.version, 4519022873219374);
    assert!(release.runtime_available);
    assert_eq!(release.serving.unwrap().ready, Some(1));
    let progress = release.progress.unwrap();
    assert_eq!(progress.steps[0].name, "Wait for rollout");
    assert_eq!(release.target.unwrap().plan.as_deref(), Some("surge"));
}

#[tokio::test]
async fn release_degraded_runtime_has_null_counts() {
    let server = MockServer::start().await;
    let mut body = full_release();
    body["runtime_available"] = json!(false);
    body["serving"]["ready"] = json!(null);
    body["serving"]["desired"] = json!(null);
    mock_release(&server, body).await;
    let release = client(&server)
        .get_deployment_release("org_1", "dep_1")
        .await
        .unwrap();
    assert!(!release.runtime_available);
    let serving = release.serving.unwrap();
    assert_eq!((serving.ready, serving.desired), (None, None));
}

#[tokio::test]
async fn release_defaults_runtime_available_and_tolerates_unknown_phase() {
    let server = MockServer::start().await;
    let mut body = full_release();
    body["phase"] = json!("paused_by_gremlins");
    body.as_object_mut().unwrap().remove("runtime_available");
    mock_release(&server, body).await;
    let release = client(&server)
        .get_deployment_release("org_1", "dep_1")
        .await
        .unwrap();
    assert_eq!(release.phase, ReleasePhase::Unknown);
    assert!(release.runtime_available);
}

#[tokio::test]
async fn deploy_with_allow_outage_sends_the_flag() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/deploy")))
        .and(body_json(json!({"mode": "force", "allow_outage": true})))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "queued": true, "app_id": "dep_1", "log_id": "log_1",
            "action": "deploy", "status": "queued"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let ack = client(&server)
        .deploy_with_options(
            "org_1",
            "dep_1",
            DeployOptions {
                mode: None,
                allow_outage: true,
            },
        )
        .await
        .unwrap();
    assert_eq!(ack.log_id, "log_1");
}

#[tokio::test]
async fn deploy_without_allow_outage_omits_the_flag() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/deploy")))
        .and(body_json(json!({"mode": "if_image_changed"})))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "queued": true, "app_id": "dep_1", "log_id": "log_2",
            "action": "deploy", "status": "queued"
        })))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .deploy_with_options(
            "org_1",
            "dep_1",
            DeployOptions {
                mode: Some("if_image_changed".into()),
                allow_outage: false,
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn needs_outage_conflict_surfaces_code_and_details() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/deploy")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "status": 409, "error": "rollout_needs_outage", "code": "rollout_needs_outage",
            "message": "There is no room on the cluster",
            "details": {
                "serving_image": "registry.example/team/app@sha256:aa",
                "needed": {"cpu_millicores": 250, "memory_mib": 256},
                "best_free": {"cpu_millicores": 100, "memory_mib": 180},
                "eligible_nodes": 3, "outage_available": true, "startup_probe_seconds": 30
            }
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .deploy_with_options("org_1", "dep_1", DeployOptions::default())
        .await
        .unwrap_err();
    assert_eq!(err.api_status(), Some(409));
    assert_eq!(err.api_code(), Some("rollout_needs_outage"));
    assert_eq!(err.api_details().unwrap()["eligible_nodes"], 3);
    let details = err.rollout_capacity_details().unwrap();
    assert_eq!(details.needed.unwrap().cpu_millicores, 250);
    assert_eq!(details.startup_probe_seconds, Some(30));
    assert!(details.outage_available);
}

#[tokio::test]
async fn conflict_without_details_stays_a_plain_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/deploy")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "code": "busy", "message": "a deploy is already running"
        })))
        .mount(&server)
        .await;
    let err = client(&server)
        .deploy_with_options("org_1", "dep_1", DeployOptions::default())
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Api { status: 409, .. }));
    assert_eq!(err.api_code(), Some("busy"));
    assert!(err.api_details().is_none());
}

#[tokio::test]
async fn status_new_fields_are_optional_and_parsed_when_present() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{BASE}/status")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "running": true, "ready_replicas": 1, "desired_replicas": 1,
            "message": "ok", "db_status": "running"
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let old = client(&server)
        .get_deployment_status("org_1", "dep_1")
        .await
        .unwrap();
    assert!(old.serving_replicas.is_none() && old.rollout.is_none() && old.release.is_none());

    Mock::given(method("GET"))
        .and(path(format!("{BASE}/status")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "running": true, "ready_replicas": 1, "desired_replicas": 1,
            "message": "ok", "db_status": "running", "serving_replicas": 1,
            "rollout": {"complete": false, "reason": "waiting", "desired": 1, "updated": 1,
                        "updated_ready": 0, "total": 2},
            "release": full_release()
        })))
        .mount(&server)
        .await;
    let new = client(&server)
        .get_deployment_status("org_1", "dep_1")
        .await
        .unwrap();
    assert_eq!(new.serving_replicas, Some(1));
    assert_eq!(new.rollout.unwrap().total, 2);
    assert_eq!(new.release.unwrap().phase, ReleasePhase::RollingOut);
}

async fn error_for(status: u16, body: serde_json::Value) -> CopepodError {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{BASE}/deploy")))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .mount(&server)
        .await;
    client(&server)
        .deploy_with_options("org_1", "dep_1", DeployOptions::default())
        .await
        .unwrap_err()
}

#[tokio::test]
async fn record_validation_error_with_details_stays_api() {
    let err = error_for(
        422,
        json!({"status": 422, "error": "immutable_field", "code": "immutable_field",
               "message": "field 'owner' cannot be changed", "field": "owner",
               "details": {"field": "owner"}}),
    )
    .await;
    assert!(matches!(
        err,
        CopepodError::Api { status: 422, code: Some(ref c), .. } if c == "immutable_field"
    ));
    assert!(err.api_details().is_none());
}

#[tokio::test]
async fn quota_error_with_details_stays_api() {
    let err = error_for(
        422,
        json!({"status": 422, "error": "storage_quota_exceeded", "code": "storage_quota_exceeded",
               "message": "Storage quota exceeded: 5 GB of 5 GB used.",
               "used_bytes": 5, "limit_bytes": 5, "file_bytes": 1,
               "details": {"used_bytes": 5}}),
    )
    .await;
    assert!(matches!(
        err,
        CopepodError::Api { status: 422, code: Some(ref c), .. } if c == "storage_quota_exceeded"
    ));
    assert!(err.api_details().is_none());
}
