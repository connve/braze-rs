//! Offline smoke tests for the Export API.
//!
//! These use `wiremock` to stub `POST /users/export/ids` so contributors can
//! exercise the wrapper → HTTP → serde path without a live Braze account.
//! They complement `tests/export.rs`, which hits the real API and is skipped
//! when `BRAZE_CREDENTIALS` is unset.

use braze::export::ExportUsersByIdsRequest;
use braze::{Client, Credentials};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn users_by_ids_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "external_ids": ["user_1"],
        "fields_to_export": ["email", "first_name"],
    });

    let response_body = json!({
        "message": "success",
        "users": [
            {
                "external_id": "user_1",
                "email": "user_1@example.com",
                "first_name": "Ada",
            }
        ],
        "invalid_user_ids": [],
    });

    Mock::given(method("POST"))
        .and(path("/users/export/ids"))
        .and(header("authorization", "Bearer fake-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .credentials(Credentials {
            api_key: "fake-key".into(),
            rest_endpoint: server.uri(),
        })
        .build()?;

    let response = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest {
            external_ids: Some(vec!["user_1".into()]),
            fields_to_export: Some(vec!["email".into(), "first_name".into()]),
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.message, "success");
    assert_eq!(response.users.len(), 1);
    let user = &response.users[0];
    assert_eq!(user.external_id.as_deref(), Some("user_1"));
    assert_eq!(user.email.as_deref(), Some("user_1@example.com"));
    assert_eq!(user.first_name.as_deref(), Some("Ada"));
    assert!(response.invalid_user_ids.is_empty());

    Ok(())
}

#[tokio::test]
async fn users_by_ids_surfaces_api_error_body() -> Result {
    let server = MockServer::start().await;

    let error_body: Value = json!({
        "message": "Bad Request",
        "errors": ["at least one identifier is required"],
    });

    Mock::given(method("POST"))
        .and(path("/users/export/ids"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&error_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .credentials(Credentials {
            api_key: "fake-key".into(),
            rest_endpoint: server.uri(),
        })
        .build()?;

    let err = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest::default())
        .send()
        .await
        .expect_err("400 response must be an error");

    assert!(!err.is_retryable(), "400 must not be retryable");
    match err {
        braze::Error::Api { status, body } => {
            assert_eq!(status.as_u16(), 400);
            assert!(body.contains("at least one identifier is required"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    Ok(())
}
