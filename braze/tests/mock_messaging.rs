//! Offline tests for the messaging endpoints.
//!
//! Runs against a local `wiremock` server and needs nothing else, so it
//! isn't `#[ignore]`d.

use braze::{
    header::{HeaderMap, HeaderValue},
    messaging::{
        CanvasRecipient, CreateSendIdsRequest, Messages, Recipient, SendMessagesRequest,
        SendMessagesRequestRecipientSubscriptionState, SendTransactionalMessageRequest,
        SendTriggeredCampaignsRequest, SendTriggeredCanvasesRequest, TransactionalRecipient,
        UserAlias,
    },
    Client,
};
use serde_json::{json, Value};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn send_messages_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "broadcast": false,
        "external_user_ids": ["user_1"],
        "messages": {
            "email": {
                "body": "World",
                "subject": "Hello"
            }
        },
        "override_frequency_capping": false,
        "recipient_subscription_state": "opted_in"
    });

    let response_body = json!({
        "message": "success",
        "dispatch_id": "dispatch-123"
    });

    Mock::given(method("POST"))
        .and(path("/messages/send"))
        .and(header("authorization", "Bearer fake-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let mut messages = Messages::default();
    messages.email.insert("subject".into(), "Hello".into());
    messages.email.insert("body".into(), "World".into());

    let response = client
        .messaging()
        .send_messages(&SendMessagesRequest {
            external_user_ids: vec!["user_1".into()],
            recipient_subscription_state: Some(
                SendMessagesRequestRecipientSubscriptionState::OptedIn,
            ),
            messages: Some(messages),
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.message.as_deref(), Some("success"));
    assert_eq!(response.dispatch_id.as_deref(), Some("dispatch-123"));

    Ok(())
}

#[tokio::test]
async fn send_triggered_campaigns_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "campaign_id": "campaign-123",
        "broadcast": true
    });

    let response_body = json!({
        "message": "success",
        "dispatch_id": "dispatch-456",
        "notice": "campaign sent"
    });

    Mock::given(method("POST"))
        .and(path("/campaigns/trigger/send"))
        .and(header("authorization", "Bearer fake-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let response = client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id: "campaign-123".into(),
            broadcast: true,
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.message.as_deref(), Some("success"));
    assert_eq!(response.dispatch_id.as_deref(), Some("dispatch-456"));
    assert_eq!(response.notice.as_deref(), Some("campaign sent"));

    Ok(())
}

#[tokio::test]
async fn send_triggered_canvases_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "broadcast": false,
        "canvas_id": "canvas-123",
        "recipients": [
            {
                "external_user_id": "user_1",
                "send_to_existing_only": true
            }
        ]
    });

    let response_body = json!({
        "message": "success",
        "dispatch_id": "dispatch-789"
    });

    Mock::given(method("POST"))
        .and(path("/canvas/trigger/send"))
        .and(header("authorization", "Bearer fake-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let response = client
        .messaging()
        .send_triggered_canvases(&SendTriggeredCanvasesRequest {
            canvas_id: "canvas-123".into(),
            recipients: vec![CanvasRecipient {
                external_user_id: Some("user_1".into()),
                send_to_existing_only: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.message.as_deref(), Some("success"));
    assert_eq!(response.dispatch_id.as_deref(), Some("dispatch-789"));

    Ok(())
}

#[tokio::test]
async fn create_send_ids_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "campaign_id": "campaign-123",
        "send_id": "send-123"
    });

    let response_body = json!({
        "message": "success",
        "send_id": "send-123"
    });

    Mock::given(method("POST"))
        .and(path("/sends/id/create"))
        .and(header("authorization", "Bearer fake-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let response = client
        .messaging()
        .create_send_ids(&CreateSendIdsRequest {
            campaign_id: "campaign-123".into(),
            send_id: Some("send-123".into()),
        })
        .send()
        .await?;

    assert_eq!(response.message.as_deref(), Some("success"));
    assert_eq!(response.send_id.as_deref(), Some("send-123"));

    Ok(())
}

#[tokio::test]
async fn send_transactional_message_sends_expected_request() -> Result {
    let server = MockServer::start().await;

    let expected_body = json!({
        "external_send_id": "external-123",
        "recipient": {
            "external_user_id": "user_1"
        }
    });

    let response_body = json!({
        "dispatch_id": "dispatch-abc",
        "status": "sent",
        "metadata": {
            "external_send_id": "external-123"
        }
    });

    Mock::given(method("POST"))
        .and(path("/transactional/v1/campaigns/campaign-123/send"))
        .and(header("authorization", "Bearer fake-key"))
        .and(header("content-type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let response = client
        .messaging()
        .send_transactional_message(
            "campaign-123",
            &SendTransactionalMessageRequest {
                external_send_id: Some("external-123".into()),
                recipient: TransactionalRecipient {
                    external_user_id: Some("user_1".into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .send()
        .await?;

    assert_eq!(response.dispatch_id.as_deref(), Some("dispatch-abc"));
    assert_eq!(response.status.as_deref(), Some("sent"));

    Ok(())
}

#[tokio::test]
async fn send_transactional_message_encodes_campaign_id() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/transactional/v1/campaigns/a%2F..%2Fb/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "dispatch_id": "d" })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let response = client
        .messaging()
        .send_transactional_message("a/../b", &SendTransactionalMessageRequest::default())
        .send()
        .await?;

    assert_eq!(response.dispatch_id.as_deref(), Some("d"));

    Ok(())
}

#[tokio::test]
async fn campaign_id_that_url_would_rewrite_is_rejected_without_a_request() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    for campaign_id in ["", ".", "..", "..\t"] {
        let err = client
            .messaging()
            .send_transactional_message(campaign_id, &SendTransactionalMessageRequest::default())
            .send()
            .await
            .expect_err("campaign_id must be rejected");
        assert!(
            matches!(err, braze::Error::InvalidPathSegment { ref segment } if segment == campaign_id),
            "{campaign_id:?}: got {err:?}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn alias_recipient_omits_send_to_existing_only() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/campaigns/trigger/send"))
        .and(body_json(json!({
            "campaign_id": "campaign-123",
            "broadcast": false,
            "recipients": [
                { "user_alias": { "alias_name": "a", "alias_label": "l" } }
            ]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id: "campaign-123".into(),
            recipients: vec![Recipient {
                user_alias: Some(UserAlias {
                    alias_name: "a".into(),
                    alias_label: "l".into(),
                }),
                ..Default::default()
            }],
            ..Default::default()
        })
        .send()
        .await?;

    Ok(())
}

#[tokio::test]
async fn send_messages_keeps_non_fatal_errors_and_unknown_fields() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages/send"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "message": "success",
            "dispatch_id": "d",
            "errors": ["invalid external_id x"],
            "new_braze_field": 42
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let response = client
        .messaging()
        .send_messages(&SendMessagesRequest::default())
        .send()
        .await?;

    assert_eq!(response.errors, [json!("invalid external_id x")]);
    assert_eq!(response.extra.get("new_braze_field"), Some(&json!(42)));

    Ok(())
}

#[tokio::test]
async fn per_call_headers_merge_over_client_defaults() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/sends/id/create"))
        .and(header("x-tenant", "default"))
        .and(header("x-request-id", "per-call"))
        .and(header("x-trace-id", "kept"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "send_id": "s" })))
        .expect(1)
        .mount(&server)
        .await;

    let mut defaults = HeaderMap::new();
    defaults.insert("x-tenant", HeaderValue::from_static("default"));
    defaults.insert("x-request-id", HeaderValue::from_static("client-default"));

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .default_headers(defaults)
        .build()?;

    let mut per_call = HeaderMap::new();
    per_call.insert("x-request-id", HeaderValue::from_static("per-call"));

    let response = client
        .messaging()
        .create_send_ids(&CreateSendIdsRequest {
            campaign_id: "campaign-123".into(),
            send_id: None,
        })
        .header("X-Trace-Id", "kept")
        .headers(per_call)
        .send()
        .await?;

    assert_eq!(response.send_id.as_deref(), Some("s"));

    Ok(())
}

#[tokio::test]
async fn reserved_per_call_header_is_rejected_without_a_request() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let err = client
        .messaging()
        .create_send_ids(&CreateSendIdsRequest {
            campaign_id: "campaign-123".into(),
            send_id: None,
        })
        .header("Authorization", "Bearer someone-else")
        .send()
        .await
        .expect_err("reserved header must be rejected");

    assert!(!err.is_retryable());
    assert!(matches!(
        err,
        braze::Error::InvalidHeader { ref name } if name == "authorization"
    ));

    Ok(())
}

#[tokio::test]
async fn messaging_surfaces_api_error_body() -> Result {
    let server = MockServer::start().await;

    let error_body: Value = json!({
        "message": "Bad Request",
        "errors": ["campaign_id is required"]
    });

    Mock::given(method("POST"))
        .and(path("/campaigns/trigger/send"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&error_body))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let err = client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id: "campaign-123".into(),
            ..Default::default()
        })
        .send()
        .await
        .expect_err("400 response must be an error");

    assert!(!err.is_retryable(), "400 must not be retryable");
    match err {
        braze::Error::Api { status, body, .. } => {
            assert_eq!(status.as_u16(), 400);
            assert!(body.contains("campaign_id is required"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }

    Ok(())
}

#[tokio::test]
async fn rate_limited_request_reports_retry_after() -> Result {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/campaigns/trigger/send"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("x-ratelimit-retry-after", "30")
                .set_body_json(json!({ "message": "Rate limit exceeded" })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .api_key("fake-key")
        .base_url(server.uri())
        .build()?;

    let err = client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id: "campaign-123".into(),
            ..Default::default()
        })
        .send()
        .await
        .expect_err("429 response must be an error");

    assert!(err.is_retryable());
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(30)));

    Ok(())
}
