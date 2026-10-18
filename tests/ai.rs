use copepod_sdk::{AiTranscriptionRequest, CopepodClient, CopepodError};
use futures_util::StreamExt;
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const U: &str = "/api/platform/orgs/org/apps/app/auth/users/me/ai/v1";
const K: &str = "/api/platform/orgs/org/apps/app/ai/v1";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .token("user-bearer")
        .api_key("app-key")
        .auto_refresh(false)
        .build()
        .unwrap()
}

fn no_authorization(req: &Request) -> bool {
    !req.headers.contains_key("authorization")
}

#[tokio::test]
async fn chat_sends_bearer_and_idempotency_key_and_parses_credit_headers() {
    let server = MockServer::start().await;
    let body =
        json!({"model": "mistral-small-latest", "messages": [{"role": "user", "content": "hi"}]});
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "k-1"))
        .and(body_json(&body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Copepod-Request-Id", "gw:k-1")
                .insert_header("X-Copepod-Credits-Charged", "3")
                .insert_header("X-Copepod-Credits-Remaining", "247")
                .insert_header("X-Copepod-Usage-Estimated", "true")
                .set_body_json(json!({"choices": []})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_chat("org", "app", "users", &body, Some("k-1"))
        .await
        .unwrap();
    assert_eq!(r.body, json!({"choices": []}));
    assert_eq!(r.request_id.as_deref(), Some("gw:k-1"));
    assert_eq!(r.charged_credits, Some(3));
    assert_eq!(r.remaining_credits, Some(247));
    assert!(r.estimated);
}

#[tokio::test]
async fn missing_credit_headers_are_none() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/embeddings")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_embeddings("org", "app", "users", &json!({"input": ["a"]}), None)
        .await
        .unwrap();
    assert_eq!(r.request_id, None);
    assert_eq!(r.charged_credits, None);
    assert!(!r.estimated);
}

#[tokio::test]
async fn credits_exhausted_maps_to_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "status": 402, "code": "ai_credits_exhausted",
            "error": "ai_credits_exhausted", "message": "No credits left",
            "remaining_total": 0, "resets_at": "2026-11-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    let err = client(&server)
        .ai_chat("org", "app", "users", &json!({"messages": []}), None)
        .await
        .unwrap_err();
    match err {
        CopepodError::ApiWithDetails {
            status,
            ref code,
            ref details,
            ..
        } => {
            assert_eq!(status, 402);
            assert_eq!(code.as_deref(), Some("ai_credits_exhausted"));
            assert_eq!(details["remaining_total"], 0);
            assert_eq!(details["resets_at"], "2026-11-01T00:00:00Z");
        }
        ref other => panic!("unexpected {other:?}"),
    }
    assert_eq!(err.api_code(), Some("ai_credits_exhausted"));
    assert_eq!(err.api_status(), Some(402));
}

#[tokio::test]
async fn replay_conflict_and_model_not_allowed_keep_their_codes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .and(header("idempotency-key", "dup"))
        .respond_with(ResponseTemplate::new(409).set_body_json(
            json!({"status": 409, "code": "ai_request_replayed", "error": "ai_request_replayed",
                "message": "replayed", "charged_credits": 7}),
        ))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/embeddings")))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(
                json!({"status": 403, "code": "ai_model_not_allowed", "message": "no"}),
            ),
        )
        .mount(&server)
        .await;
    let c = client(&server);
    let e = c
        .ai_chat("org", "app", "users", &json!({}), Some("dup"))
        .await
        .unwrap_err();
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(409), Some("ai_request_replayed"))
    );
    assert_eq!(e.api_details().unwrap()["charged_credits"], 7);
    let e = c
        .ai_embeddings("org", "app", "users", &json!({"model": "x"}), None)
        .await
        .unwrap_err();
    assert_eq!(
        (e.api_status(), e.api_code()),
        (Some(403), Some("ai_model_not_allowed"))
    );
}

#[tokio::test]
async fn limiter_unavailable_503_is_an_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/ocr")))
        .respond_with(
            ResponseTemplate::new(503).set_body_json(
                json!({"status": 503, "message": "shared rate limiter unavailable"}),
            ),
        )
        .mount(&server)
        .await;
    let e = client(&server)
        .ai_ocr("org", "app", "users", &json!({"document": {}}), None)
        .await
        .unwrap_err();
    assert_eq!(e.api_status(), Some(503));
    assert_eq!(e.api_code(), None);
}

#[tokio::test]
async fn coded_limiter_unavailable_503_keeps_its_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/ocr")))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "status": 503, "code": "ai_rate_limiter_unavailable",
            "error": "ai_rate_limiter_unavailable",
            "message": "shared rate limiter unavailable"
        })))
        .mount(&server)
        .await;
    let e = client(&server)
        .ai_ocr("org", "app", "users", &json!({"document": {}}), None)
        .await
        .unwrap_err();
    assert_eq!(e.api_status(), Some(503));
    assert_eq!(e.api_code(), Some("ai_rate_limiter_unavailable"));
}

#[tokio::test]
async fn ocr_hits_its_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/ocr")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(body_json(json!({"model": "mistral-ocr-latest"})))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Copepod-Credits-Charged", "2")
                .set_body_json(json!({"pages": []})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_ocr(
            "org",
            "app",
            "users",
            &json!({"model": "mistral-ocr-latest"}),
            None,
        )
        .await
        .unwrap();
    assert_eq!(r.charged_credits, Some(2));
}

#[tokio::test]
async fn stream_yields_three_chunks_until_done() {
    let server = MockServer::start().await;
    let sse = "data: {\"n\":1}\n\ndata: {\"n\":2}\n\ndata: {\"n\":3}\n\ndata: [DONE]\n\ndata: {\"n\":4}\n\n";
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "s-1"))
        .and(body_json(json!({"model": "m", "stream": true})))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Copepod-Request-Id", "gw:s-1")
                .set_body_raw(sse, "text/event-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let mut stream = client(&server)
        .ai_chat_stream("org", "app", "users", &json!({"model": "m"}), Some("s-1"))
        .await
        .unwrap();
    let mut seen = vec![];
    while let Some(item) = stream.next().await {
        seen.push(item.unwrap()["n"].as_i64().unwrap());
    }
    assert_eq!(seen, vec![1, 2, 3]);
}

#[tokio::test]
async fn stream_error_event_becomes_an_api_error_and_ends() {
    let server = MockServer::start().await;
    let sse = "data: {\"n\":1}\n\ndata: {\"error\":{\"code\":\"ai_provider_unavailable\",\"message\":\"upstream died\"}}\n\ndata: {\"n\":2}\n\n";
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse, "text/event-stream"))
        .mount(&server)
        .await;
    let mut stream = client(&server)
        .ai_chat_stream("org", "app", "users", &json!({}), None)
        .await
        .unwrap();
    assert_eq!(stream.next().await.unwrap().unwrap()["n"], 1);
    let err = stream.next().await.unwrap().unwrap_err();
    assert_eq!(err.api_code(), Some("ai_provider_unavailable"));
    assert_eq!(err.api_status(), Some(503));
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn stream_start_failure_is_mapped_before_any_chunk() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .respond_with(ResponseTemplate::new(402).set_body_json(
            json!({"status": 402, "code": "ai_credits_exhausted", "message": "none"}),
        ))
        .mount(&server)
        .await;
    let err = client(&server)
        .ai_chat_stream("org", "app", "users", &json!({}), None)
        .await
        .err()
        .unwrap();
    assert_eq!(err.api_code(), Some("ai_credits_exhausted"));
}

#[tokio::test]
async fn upload_sends_multipart_with_purpose_ocr() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/files")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header_exists("content-type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "f1"})))
        .expect(1)
        .mount(&server)
        .await;
    let v = client(&server)
        .ai_upload_file(
            "org",
            "app",
            "users",
            b"%PDF".to_vec(),
            "a.pdf",
            Some("up-1"),
        )
        .await
        .unwrap();
    assert_eq!(v["id"], "f1");
    let req = &server.received_requests().await.unwrap()[0];
    let body = String::from_utf8_lossy(&req.body);
    assert!(req.headers["content-type"]
        .to_str()
        .unwrap()
        .starts_with("multipart/form-data"));
    assert!(body.contains("name=\"purpose\"") && body.contains("ocr"));
    assert!(body.contains("name=\"file\"; filename=\"a.pdf\""));
}

#[tokio::test]
async fn file_url_and_delete_hit_their_paths() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("{U}/files/f1/url")))
        .and(query_param("expiry", "6"))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"url": "https://x"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{U}/files/f1")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted": true})))
        .expect(1)
        .mount(&server)
        .await;
    let c = client(&server);
    assert_eq!(
        c.ai_file_url("org", "app", "users", "f1", 6).await.unwrap()["url"],
        "https://x"
    );
    c.ai_delete_file("org", "app", "users", "f1").await.unwrap();
}

#[tokio::test]
async fn delete_missing_file_maps_404() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("{U}/files/gone")))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(
                json!({"status": 404, "code": "ai_file_not_found", "message": "no"}),
            ),
        )
        .mount(&server)
        .await;
    let e = client(&server)
        .ai_delete_file("org", "app", "users", "gone")
        .await
        .unwrap_err();
    assert_eq!(e.api_code(), Some("ai_file_not_found"));
}

#[tokio::test]
async fn transcribe_sends_model_fields_and_file() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/audio/transcriptions")))
        .and(header("authorization", "Bearer user-bearer"))
        .and(header("idempotency-key", "tr-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-Copepod-Credits-Charged", "5")
                .set_body_json(json!({"text": "hello"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let r = client(&server)
        .ai_transcribe(
            "org",
            "app",
            "users",
            AiTranscriptionRequest {
                model: "voxtral-mini-latest".into(),
                bytes: vec![1, 2, 3],
                filename: "a.wav".into(),
                fields: vec![("language".into(), "en".into())],
            },
            Some("tr-1"),
        )
        .await
        .unwrap();
    assert_eq!(r.body["text"], "hello");
    assert_eq!(r.charged_credits, Some(5));
    let req = &server.received_requests().await.unwrap()[0];
    let body = String::from_utf8_lossy(&req.body);
    assert!(body.contains("name=\"model\"") && body.contains("voxtral-mini-latest"));
    assert!(body.contains("name=\"language\""));
    assert!(body.contains("filename=\"a.wav\""));
}

#[tokio::test]
async fn user_moderations_use_bearer_on_both_routes() {
    let server = MockServer::start().await;
    for route in ["moderations", "chat/moderations"] {
        Mock::given(method("POST"))
            .and(path(format!("{U}/{route}")))
            .and(header("authorization", "Bearer user-bearer"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": []})))
            .expect(1)
            .mount(&server)
            .await;
    }
    let c = client(&server);
    let body = json!({"model": "mistral-moderation-latest", "input": "x"});
    c.ai_moderations("org", "app", "users", &body)
        .await
        .unwrap();
    c.ai_chat_moderations("org", "app", "users", &body)
        .await
        .unwrap();
}

#[tokio::test]
async fn app_moderations_send_api_key_without_authorization() {
    let server = MockServer::start().await;
    for (route, user) in [("moderations", Some("u1")), ("chat/moderations", None)] {
        let mock = Mock::given(method("POST"))
            .and(path(format!("{K}/{route}")))
            .and(header("x-api-key", "app-key"))
            .and(no_authorization);
        let mock = if user.is_some() {
            mock.and(header("x-user-id", "u1"))
        } else {
            mock
        };
        mock.respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": []})))
            .expect(1)
            .mount(&server)
            .await;
    }
    let c = client(&server);
    let body = json!({"model": "mistral-moderation-latest", "input": "x"});
    c.app_ai_moderations("org", "app", &body, false, Some("u1"))
        .await
        .unwrap();
    c.app_ai_moderations("org", "app", &body, true, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn app_moderations_need_an_api_key() {
    let c = CopepodClient::builder()
        .base_url("http://127.0.0.1:1")
        .token("t")
        .auto_refresh(false)
        .build()
        .unwrap();
    let e = c
        .app_ai_moderations("org", "app", &json!({"input": "x"}), false, None)
        .await
        .unwrap_err();
    assert!(matches!(e, CopepodError::Auth(_)));
}

#[tokio::test]
async fn invalid_arguments_fail_before_any_request() {
    let server = MockServer::start().await;
    let c = client(&server);
    let bad = |e: CopepodError| matches!(e, CopepodError::InvalidArgument(_));
    assert!(bad(c
        .ai_chat("org", "app", "users", &json!([1]), None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_chat("org", "app", "users", &json!({"model": 3}), None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_chat("org", "app", "users", &json!({}), Some("bad key!"))
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_chat("org", "app", "users", &json!({}), Some(&"a".repeat(161)))
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_file_url("org", "app", "users", "f", 0)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_file_url("org", "app", "users", "f", 25)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_file_url("org", "app", "users", "a/b", 1)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_upload_file("org", "app", "users", vec![], " ", None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_chat_stream("org", "app", "users", &json!("x"), None)
        .await
        .err()
        .unwrap()));
    let req = AiTranscriptionRequest {
        model: "m".into(),
        bytes: vec![],
        filename: "a.wav".into(),
        fields: vec![("file".into(), "x".into())],
    };
    assert!(bad(c
        .ai_transcribe("org", "app", "users", req, None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_chat("org", "app", "users", &json!({"stream": true}), None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_chat("org", "app", "users", &json!({"model": ""}), None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_file_url("org", "app", "users", "a.b", 1)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_file_url("org", "app", "users", &"a".repeat(129), 1)
        .await
        .unwrap_err()));
    let streaming = AiTranscriptionRequest {
        model: "m".into(),
        bytes: vec![],
        filename: "a.wav".into(),
        fields: vec![("stream".into(), "true".into())],
    };
    assert!(bad(c
        .ai_transcribe("org", "app", "users", streaming, None)
        .await
        .unwrap_err()));
    assert!(bad(c
        .ai_ocr("org", "app", "users", &json!({}), Some("no good"))
        .await
        .unwrap_err()));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn null_model_and_stream_false_are_accepted() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    client(&server)
        .ai_chat(
            "org",
            "app",
            "users",
            &json!({"model": null, "stream": false}),
            None,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn metered_methods_send_the_idempotency_key() {
    let server = MockServer::start().await;
    for route in ["embeddings", "ocr", "files"] {
        Mock::given(method("POST"))
            .and(path(format!("{U}/{route}")))
            .and(header("idempotency-key", "k-9"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .expect(1)
            .mount(&server)
            .await;
    }
    let c = client(&server);
    c.ai_embeddings("org", "app", "users", &json!({}), Some("k-9"))
        .await
        .unwrap();
    c.ai_ocr("org", "app", "users", &json!({}), Some("k-9"))
        .await
        .unwrap();
    c.ai_upload_file("org", "app", "users", vec![1], "a.pdf", Some("k-9"))
        .await
        .unwrap();
}

#[tokio::test]
async fn stream_closed_before_done_yields_one_sse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw("data: {\"n\":1}\n\n", "text/event-stream"),
        )
        .mount(&server)
        .await;
    let mut stream = client(&server)
        .ai_chat_stream("org", "app", "users", &json!({}), None)
        .await
        .unwrap();
    assert_eq!(stream.next().await.unwrap().unwrap()["n"], 1);
    match stream.next().await.unwrap() {
        Err(CopepodError::Sse(message)) => assert_eq!(message, "stream ended before [DONE]"),
        other => panic!("unexpected {other:?}"),
    }
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn urls_and_scoped_client() {
    let server = MockServer::start().await;
    let c = client(&server);
    assert_eq!(
        c.ai_gateway_base_url("org", "app", "users"),
        format!(
            "{}/api/platform/orgs/org/apps/app/auth/users/me/ai",
            server.uri()
        )
    );
    let ws = c
        .ai_realtime_url("org", "app", "users", "voxtral-realtime")
        .unwrap();
    assert!(ws.starts_with("ws://"));
    assert!(
        ws.ends_with("/auth/users/me/ai/v1/audio/transcriptions/realtime?model=voxtral-realtime")
    );
    assert!(!ws.contains("user-bearer"));
    assert!(matches!(
        c.ai_realtime_url("org", "app", "users", ""),
        Err(CopepodError::InvalidArgument(_))
    ));

    Mock::given(method("POST"))
        .and(path(format!("{U}/chat/completions")))
        .and(header("authorization", "Bearer user-bearer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;
    let r = c
        .app("org", "app")
        .ai("users")
        .chat(&json!({}), None)
        .await
        .unwrap();
    assert_eq!(r.body["ok"], true);
}
