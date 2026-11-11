use copepod_sdk::{
    AnnouncementSeverity, AppRegisterRequest, CopepodClient, CopepodError, CreateLeadRequest,
    LeadAcquisition, MarketingConsent, RegisterWithBillingIntentRequest,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const LEADS: &str = "/api/platform/orgs/org_1/apps/app_1/leads";
const ACTIVE: &str = "/api/platform/orgs/org_1/apps/app_1/announcements/active";

fn client(server: &MockServer) -> CopepodClient {
    CopepodClient::builder()
        .base_url(server.uri())
        .api_key("app-secret")
        .build()
        .unwrap()
}

fn lead() -> CreateLeadRequest {
    CreateLeadRequest {
        list_key: "launch_waitlist".into(),
        email: "Ada@Example.com".into(),
        consent_text: "Email me when it launches.".into(),
        country: Some("GB".into()),
        acquisition: Some(LeadAcquisition {
            utm_source: Some("hn".into()),
            referral: Some("friend".into()),
            landing_path: Some("/pricing".into()),
            ..Default::default()
        }),
        client_ip: Some("203.0.113.9".into()),
        turnstile_token: Some("0.xxxx".into()),
    }
}

#[tokio::test]
async fn create_lead_uses_api_key_and_exact_body_without_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(LEADS))
        .and(header("X-API-Key", "app-secret"))
        .and(body_json(json!({
            "list_key": "launch_waitlist",
            "email": "Ada@Example.com",
            "consent_text": "Email me when it launches.",
            "country": "GB",
            "acquisition": {"utm_source": "hn", "ref": "friend", "landing_path": "/pricing"},
            "client_ip": "203.0.113.9",
            "turnstile_token": "0.xxxx"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"accepted": true})))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server)
        .create_lead("org_1", "app_1", &lead())
        .await
        .unwrap();
    assert!(response.accepted);
    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn create_lead_maps_error_codes() {
    // (status, code, expected code in the SDK error); bodies use the server's
    // real shape {status, code, error, message}.
    let cases: [(u16, Option<&str>); 4] = [
        (429, Some("rate_limited")),
        (422, Some("turnstile_failed")),
        (503, Some("audience_upgrading")),
        (403, None),
    ];
    for (status, code) in cases {
        let server = MockServer::start().await;
        let body = match code {
            Some(code) => json!({
                "status": status, "code": code, "error": code,
                "message": format!("{code}: server text")
            }),
            None => json!({"error": "forbidden"}),
        };
        Mock::given(method("POST"))
            .and(path(LEADS))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .mount(&server)
            .await;
        match client(&server).create_lead("org_1", "app_1", &lead()).await {
            Err(CopepodError::Api {
                status: got,
                code: got_code,
                message,
            }) => {
                assert_eq!(got, status);
                assert_eq!(got_code.as_deref(), code, "status {status}");
                assert!(!message.is_empty());
            }
            other => panic!("status {status}: expected Api, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn create_lead_validates_before_network() {
    let server = MockServer::start().await;
    let c = client(&server);
    let bad = [
        CreateLeadRequest {
            list_key: "Bad Key".into(),
            ..lead()
        },
        CreateLeadRequest {
            email: "not-an-email".into(),
            ..lead()
        },
        CreateLeadRequest {
            consent_text: "".into(),
            ..lead()
        },
        CreateLeadRequest {
            consent_text: "x".repeat(501),
            ..lead()
        },
        CreateLeadRequest {
            country: Some("GBR".into()),
            ..lead()
        },
        CreateLeadRequest {
            client_ip: Some("1.2.3.4, 5.6.7.8".into()),
            ..lead()
        },
    ];
    for request in &bad {
        let err = c.create_lead("org_1", "app_1", request).await.unwrap_err();
        assert!(matches!(err, CopepodError::InvalidArgument(_)), "{err:?}");
    }
    let err = c.create_lead("org/1", "app_1", &lead()).await.unwrap_err();
    assert!(matches!(err, CopepodError::InvalidArgument(_)));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn active_announcements_parses_list_and_severity() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACTIVE))
        .and(header("X-API-Key", "app-secret"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Cache-Control", "private, max-age=60")
                .set_body_json(json!({"announcements": [
                    {"id": "a1", "message": "Down at 3", "severity": "critical",
                     "link_url": "https://example.com/s", "starts_at": "2026-10-01T00:00:00Z",
                     "ends_at": null},
                    {"id": "a2", "message": "Hi", "severity": "info",
                     "link_url": null, "starts_at": "2026-10-01T00:00:00Z",
                     "ends_at": "2026-11-01T00:00:00Z"}
                ]})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let items = client(&server)
        .list_active_announcements("org_1", "app_1")
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].severity, AnnouncementSeverity::Critical);
    assert_eq!(items[0].link_url.as_deref(), Some("https://example.com/s"));
    assert_eq!(items[1].ends_at.as_deref(), Some("2026-11-01T00:00:00Z"));
    let requests = server.received_requests().await.unwrap();
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn audience_calls_require_an_api_key() {
    let server = MockServer::start().await;
    let c = CopepodClient::builder()
        .base_url(server.uri())
        .build()
        .unwrap();
    let err = c.create_lead("org_1", "app_1", &lead()).await.unwrap_err();
    assert!(matches!(err, CopepodError::Auth(_)), "{err:?}");
    let err = c
        .list_active_announcements("org_1", "app_1")
        .await
        .unwrap_err();
    assert!(matches!(err, CopepodError::Auth(_)), "{err:?}");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn scoped_audience_client_delegates() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(LEADS))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"accepted": true})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(ACTIVE))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"announcements": []})))
        .expect(1)
        .mount(&server)
        .await;

    let c = client(&server);
    let audience = c.app("org_1", "app_1").audience();
    assert!(audience.create_lead(&lead()).await.unwrap().accepted);
    assert!(audience.active_announcements().await.unwrap().is_empty());
}

#[test]
fn register_with_billing_intent_omits_absent_consent() {
    let mut request = RegisterWithBillingIntentRequest {
        billing_intent_id: "bi_1".into(),
        email: "a@b.co".into(),
        password: "pw".into(),
        ..Default::default()
    };
    let value = serde_json::to_value(&request).unwrap();
    assert!(value.get("marketing_consent").is_none());

    request.marketing_consent = Some(MarketingConsent {
        consent_text: "Yes".into(),
    });
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["marketing_consent"], json!({"consent_text": "Yes"}));

    let app = AppRegisterRequest::default();
    assert!(serde_json::to_value(&app)
        .unwrap()
        .get("marketing_consent")
        .is_none());
}

#[tokio::test]
async fn unknown_announcement_severity_parses_as_unknown() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(ACTIVE))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"announcements": [
                {"id": "a1", "message": "New", "severity": "urgent_plus",
                 "link_url": null, "starts_at": "2026-10-01T00:00:00Z", "ends_at": null},
                {"id": "a2", "message": "Ok", "severity": "warning",
                 "link_url": null, "starts_at": "2026-10-01T00:00:00Z", "ends_at": null}
            ]})),
        )
        .mount(&server)
        .await;
    let items = client(&server)
        .list_active_announcements("org_1", "app_1")
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].severity, AnnouncementSeverity::Unknown);
    assert_eq!(items[1].severity, AnnouncementSeverity::Warning);
}

#[tokio::test]
async fn create_lead_sends_normalised_country() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(LEADS))
        .and(wiremock::matchers::body_partial_json(
            json!({"country": "GB"}),
        ))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"accepted": true})))
        .expect(1)
        .mount(&server)
        .await;
    let request = CreateLeadRequest {
        country: Some(" gb".into()),
        ..lead()
    };
    assert!(
        client(&server)
            .create_lead("org_1", "app_1", &request)
            .await
            .unwrap()
            .accepted
    );
}

#[test]
fn marketing_consent_validates_like_the_server() {
    assert!(MarketingConsent {
        consent_text: " Yes ".into()
    }
    .validate()
    .is_ok());
    for bad in ["", "  ", "a\0b"] {
        assert!(MarketingConsent {
            consent_text: bad.into()
        }
        .validate()
        .is_err());
    }
    assert!(MarketingConsent {
        consent_text: "x".repeat(501)
    }
    .validate()
    .is_err());
}
