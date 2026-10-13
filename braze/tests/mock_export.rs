//! Offline smoke tests for the Export API.
//!
//! These use `wiremock` to stub the export endpoints so contributors can
//! exercise the wrapper → HTTP → serde path without a live Braze account.
//! They complement `tests/export.rs`, which hits the real API and is skipped
//! when `BRAZE_CREDENTIALS` is unset.

use braze::chrono::{TimeZone, Utc};
use braze::export::{
    ExportOutputFormat, ExportUsersByGlobalControlGroupRequest, ExportUsersByIdsRequest,
    ExportUsersBySegmentRequest,
};
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
                "custom_attributes": null,
                "new_user_field": 42,
            }
        ],
        "invalid_user_ids": [],
        "new_top_level_field": "kept",
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
            external_ids: vec!["user_1".into()],
            fields_to_export: vec!["email".into(), "first_name".into()],
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.message.as_deref(), Some("success"));
    assert_eq!(response.users.len(), 1);
    let user = &response.users[0];
    assert_eq!(user.external_id.as_deref(), Some("user_1"));
    assert_eq!(user.email.as_deref(), Some("user_1@example.com"));
    assert_eq!(user.first_name.as_deref(), Some("Ada"));
    assert_eq!(user.custom_attributes, None);
    assert_eq!(user.extra.get("new_user_field"), Some(&json!(42)));
    assert!(response.invalid_user_ids.is_empty());
    assert_eq!(
        response.extra.get("new_top_level_field"),
        Some(&json!("kept"))
    );

    Ok(())
}

#[tokio::test]
async fn users_by_segment_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "segment_id": "segment-123",
        "fields_to_export": ["email", "first_name"],
        "custom_attributes_to_export": ["plan"],
        "output_format": "gzip",
        "user_last_updated_at": "2026-01-01T00:00:00Z",
    });

    Mock::given(method("POST"))
        .and(path("/users/export/segment"))
        .and(header("authorization", "Bearer fake-key"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "message": "success",
            "object_prefix": "prefix-1",
            "url": "https://download.example.com/export.zip",
        })))
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
        .users_by_segment(&ExportUsersBySegmentRequest {
            segment_id: "segment-123".into(),
            fields_to_export: vec!["email".into(), "first_name".into()],
            custom_attributes_to_export: vec!["plan".into()],
            output_format: Some(ExportOutputFormat::Gzip),
            user_last_updated_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.object_prefix.as_deref(), Some("prefix-1"));
    assert_eq!(
        response.url.as_deref(),
        Some("https://download.example.com/export.zip")
    );

    Ok(())
}

#[tokio::test]
async fn users_by_global_control_group_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/users/export/global_control_group"))
        .and(header("authorization", "Bearer fake-key"))
        .and(body_json(json!({
            "fields_to_export": ["email", "braze_id"],
            "callback_endpoint": "https://hooks.example.com/braze",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "message": "success",
            "object_prefix": "prefix-2",
        })))
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
        .users_by_global_control_group(&ExportUsersByGlobalControlGroupRequest {
            fields_to_export: vec!["email".into(), "braze_id".into()],
            callback_endpoint: Some("https://hooks.example.com/braze".into()),
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.object_prefix.as_deref(), Some("prefix-2"));
    assert_eq!(response.url, None);

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
