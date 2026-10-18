//! Integration tests for the messaging endpoints.
//!
//! Every test targets a campaign or Canvas ID that doesn't exist and an
//! unknown external ID with `send_to_existing_only`, so no message is ever
//! delivered. They check auth, routing, and that Braze's rejection surfaces
//! as a non-retryable `Error::Api`.
//!
//! Runs against the Braze test workspace in `BRAZE_API_KEY` /
//! `BRAZE_BASE_URL`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use braze::messaging::{
    CanvasRecipient, CreateSendIdsRequest, Recipient, SendTransactionalMessageRequest,
    SendTriggeredCampaignsRequest, SendTriggeredCanvasesRequest, TransactionalRecipient,
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn send_triggered_campaigns_rejects_unknown_campaign() -> Result {
    let client = common::client()?;

    let result = client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id: unique_name!("campaign"),
            recipients: vec![Recipient {
                external_user_id: Some(unique_name!("user")),
                send_to_existing_only: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        })
        .send()
        .await;

    ensure_rejected!(result);
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn send_triggered_canvases_rejects_unknown_canvas() -> Result {
    let client = common::client()?;

    let result = client
        .messaging()
        .send_triggered_canvases(&SendTriggeredCanvasesRequest {
            canvas_id: unique_name!("canvas"),
            recipients: vec![CanvasRecipient {
                external_user_id: Some(unique_name!("user")),
                send_to_existing_only: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        })
        .send()
        .await;

    ensure_rejected!(result);
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn create_send_ids_rejects_unknown_campaign() -> Result {
    let client = common::client()?;

    let result = client
        .messaging()
        .create_send_ids(&CreateSendIdsRequest {
            campaign_id: unique_name!("campaign"),
            ..Default::default()
        })
        .send()
        .await;

    ensure_rejected!(result);
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn send_transactional_message_rejects_unknown_campaign() -> Result {
    let client = common::client()?;

    let campaign_id = unique_name!("campaign");
    let result = client
        .messaging()
        .send_transactional_message(
            &campaign_id,
            &SendTransactionalMessageRequest {
                recipient: TransactionalRecipient {
                    external_user_id: Some(unique_name!("user")),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .send()
        .await;

    ensure_rejected!(result);
    Ok(())
}
