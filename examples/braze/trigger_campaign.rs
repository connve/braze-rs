//! Run with:
//!
//! ```bash
//! export BRAZE_CREDENTIALS=$PWD/credentials.json
//! cargo run --example trigger_campaign --features messaging -- <campaign_id> user_id_1 user_id_2
//! ```
//!
//! Sends an API-triggered campaign to existing users only. This delivers real
//! messages — point it at a test campaign.
//!
//! `credentials.json` shape:
//!
//! ```json
//! { "api_key": "...", "rest_endpoint": "https://rest.iad-01.braze.com" }
//! ```

use std::env;

use anyhow::{Context, Result};
use braze::{
    messaging::{Recipient, SendTriggeredCampaignsRequest},
    Client,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let credentials_path = env::var("BRAZE_CREDENTIALS")
        .context("set BRAZE_CREDENTIALS to a JSON file with api_key + rest_endpoint")?;

    let mut args = env::args().skip(1);
    let campaign_id = args
        .next()
        .context("pass the campaign_id as the first CLI arg")?;
    let recipients: Vec<Recipient> = args
        .map(|external_user_id| Recipient {
            external_user_id: Some(external_user_id),
            send_to_existing_only: Some(true),
            ..Default::default()
        })
        .collect();
    if recipients.is_empty() {
        anyhow::bail!("pass at least one external_id after the campaign_id");
    }

    let client = Client::builder()
        .credentials_file(&credentials_path)?
        .build()?;

    let response = client
        .messaging()
        .send_triggered_campaigns(&SendTriggeredCampaignsRequest {
            campaign_id,
            recipients,
            ..Default::default()
        })
        .send()
        .await?;

    println!("message: {:?}", response.message);
    println!("dispatch_id: {:?}", response.dispatch_id);

    Ok(())
}
