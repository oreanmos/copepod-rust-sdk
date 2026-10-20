use copepod_sdk::{AiModelsResponse, CopepodClient, CopepodError};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const P: &str = "/api/platform/orgs/org/apps/app/auth/users/me/ai/models";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .auto_refresh(false)
        .build()
        .unwrap()
}

async fn mount(server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("GET"))
        .and(path(P))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(response)
        .expect(1)
        .mount(server)
        .await;
}

fn available(r: AiModelsResponse) -> copepod_sdk::AiModelList {
    match r {
        AiModelsResponse::Available(list) => list,
        other => panic!("expected Available, got {other:?}"),
    }
}

#[tokio::test]
async fn happy_path_decodes_models_and_retired() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "hosted_ai_enabled": true,
            "default_model": "mistral-small-latest",
            "models": [{
                "id": "zai-glm-5-3", "display_name": "GLM 5.3", "vendor": "Z.ai",
                "via": "Z.ai via Mistral", "roles": ["chat", "analysis"],
                "capabilities": {"completion_chat": true, "vision": false},
                "max_context_length": 131072, "cost_tier": "high", "deprecation": null
            }],
            "retired": [{"id": "mistral-medium-latest", "display_name": "Mistral Medium",
                         "replaced_by": "zai-glm-5-3"}]
        })),
    )
    .await;
    let list = available(
        client(&server)
            .get_ai_models("org", "app", "users")
            .await
            .unwrap(),
    );
    assert!(list.hosted_ai_enabled);
    assert_eq!(list.default_model.as_deref(), Some("mistral-small-latest"));
    let m = &list.models[0];
    assert_eq!(m.id, "zai-glm-5-3");
    assert_eq!(m.display_name, "GLM 5.3");
    assert_eq!(m.via, "Z.ai via Mistral");
    assert_eq!(m.roles, ["chat", "analysis"]);
    assert!(m.capabilities.completion_chat);
    assert!(!m.capabilities.function_calling);
    assert_eq!(m.max_context_length, Some(131072));
    assert_eq!(m.cost_tier, "high");
    assert_eq!(m.deprecation, None);
    assert_eq!(list.retired[0].replaced_by.as_deref(), Some("zai-glm-5-3"));
}

#[tokio::test]
async fn hosted_ai_off_is_an_empty_available_list() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "hosted_ai_enabled": false, "default_model": null, "models": [], "retired": []
        })),
    )
    .await;
    let list = available(
        client(&server)
            .get_ai_models("org", "app", "users")
            .await
            .unwrap(),
    );
    assert!(!list.hosted_ai_enabled);
    assert!(list.models.is_empty() && list.default_model.is_none());
}

#[tokio::test]
async fn enabled_with_no_models_and_null_default_decodes() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "hosted_ai_enabled": true, "default_model": null, "models": [], "retired": []
        })),
    )
    .await;
    let list = available(
        client(&server)
            .get_ai_models("org", "app", "users")
            .await
            .unwrap(),
    );
    assert!(list.hosted_ai_enabled && list.models.is_empty() && list.default_model.is_none());
}

#[tokio::test]
async fn not_found_from_an_old_node_is_unavailable() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(404).set_body_string("not found"),
    )
    .await;
    let r = client(&server)
        .get_ai_models("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(r, AiModelsResponse::Unavailable);
}

#[tokio::test]
async fn unauthorized_is_an_api_error() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(401).set_body_json(json!({"code": "unauthorized", "message": "no"})),
    )
    .await;
    let err = client(&server)
        .get_ai_models("org", "app", "users")
        .await
        .unwrap_err();
    assert!(
        matches!(err, CopepodError::Api { status: 401, .. }),
        "{err:?}"
    );
}

#[tokio::test]
async fn unknown_roles_and_tiers_stay_strings() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "hosted_ai_enabled": true, "default_model": "x",
            "models": [{"id": "x", "roles": ["chat", "teleport"], "cost_tier": "premium",
                        "future_field": 1}],
            "future_top": true
        })),
    )
    .await;
    let list = available(
        client(&server)
            .get_ai_models("org", "app", "users")
            .await
            .unwrap(),
    );
    assert_eq!(list.models[0].roles, ["chat", "teleport"]);
    assert_eq!(list.models[0].cost_tier, "premium");
}

#[tokio::test]
async fn missing_fields_decode_to_defaults() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(200).set_body_json(json!({
            "hosted_ai_enabled": true,
            "models": [{"id": "m"}]
        })),
    )
    .await;
    let list = available(
        client(&server)
            .get_ai_models("org", "app", "users")
            .await
            .unwrap(),
    );
    assert!(list.retired.is_empty() && list.default_model.is_none());
    let m = &list.models[0];
    assert_eq!(m.id, "m");
    assert!(m.roles.is_empty() && m.display_name.is_empty() && m.cost_tier.is_empty());
    assert!(!m.capabilities.completion_chat && m.max_context_length.is_none());
}

#[tokio::test]
async fn scoped_client_models_hits_the_same_route() {
    let server = MockServer::start().await;
    mount(&server, ResponseTemplate::new(200).set_body_json(json!({}))).await;
    let c = client(&server);
    let list = available(c.app("org", "app").ai("users").models().await.unwrap());
    assert!(!list.hosted_ai_enabled && list.models.is_empty());
}

#[tokio::test]
async fn empty_ids_are_rejected_before_the_network() {
    let server = MockServer::start().await;
    let err = client(&server)
        .get_ai_models("org", "app", " ")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
}

#[tokio::test]
async fn plain_text_404_is_unavailable() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(404).set_body_string("not found"),
    )
    .await;
    let r = client(&server)
        .get_ai_models("org", "app", "users")
        .await
        .unwrap();
    assert_eq!(r, AiModelsResponse::Unavailable);
}

#[tokio::test]
async fn json_app_not_found_404_is_an_error() {
    let server = MockServer::start().await;
    mount(
        &server,
        ResponseTemplate::new(404)
            .set_body_json(json!({"status": 404, "message": "app 'app' not found"})),
    )
    .await;
    let err = client(&server)
        .get_ai_models("org", "app", "users")
        .await
        .unwrap_err();
    assert!(
        matches!(err, CopepodError::Api { status: 404, .. }),
        "{err:?}"
    );
}

#[tokio::test]
async fn path_unsafe_ids_are_rejected() {
    let server = MockServer::start().await;
    for (o, a, c) in [
        ("a/b", "app", "users"),
        ("org", "a?x", "users"),
        ("org", "app", ".."),
    ] {
        let err = client(&server).get_ai_models(o, a, c).await.unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)));
    }
}
