//! Integration tests for the Messaging API.
//!
//! Every test targets a campaign / Canvas ID that does not exist and a random
//! external ID with `send_to_existing_only`, so running them against a real
//! workspace never delivers a message. They verify auth, routing, and that
//! Braze's rejection surfaces as a non-retryable `Error::Api`.

mod common;

use braze::messaging::{
    CanvasRecipient, CreateSendIdsRequest, Recipient, SendTransactionalMessageRequest,
    SendTriggeredCampaignsRequest, SendTriggeredCanvasesRequest, TransactionalRecipient,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn send_triggered_campaigns_rejects_unknown_campaign() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let result = client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id: common::unknown_id("campaign"),
            recipients: vec![Recipient {
                external_user_id: Some(common::unknown_id("user")),
                send_to_existing_only: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        })
        .send()
        .await;

    common::assert_rejected_by_braze(result);
    Ok(())
}

#[tokio::test]
async fn send_triggered_canvases_rejects_unknown_canvas() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let result = client
        .messaging()
        .send_triggered_canvases(&SendTriggeredCanvasesRequest {
            canvas_id: common::unknown_id("canvas"),
            recipients: vec![CanvasRecipient {
                external_user_id: Some(common::unknown_id("user")),
                send_to_existing_only: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        })
        .send()
        .await;

    common::assert_rejected_by_braze(result);
    Ok(())
}

#[tokio::test]
async fn create_send_ids_rejects_unknown_campaign() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let result = client
        .messaging()
        .create_send_ids(&CreateSendIdsRequest {
            campaign_id: common::unknown_id("campaign"),
            ..Default::default()
        })
        .send()
        .await;

    common::assert_rejected_by_braze(result);
    Ok(())
}

#[tokio::test]
async fn send_transactional_message_rejects_unknown_campaign() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let campaign_id = common::unknown_id("campaign");
    let result = client
        .messaging()
        .send_transactional_message(
            &campaign_id,
            &SendTransactionalMessageRequest {
                recipient: TransactionalRecipient {
                    external_user_id: Some(common::unknown_id("user")),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .send()
        .await;

    common::assert_rejected_by_braze(result);
    Ok(())
}
