//! Run with:
//!
//! ```bash
//! export BRAZE_CREDENTIALS=$PWD/credentials.json
//! cargo run --example export_segment --features export -- <segment_id>
//! ```
//!
//! Starts an export of every user in the segment. Files go to your configured
//! storage bucket, or to the temporary `url` printed below.
//!
//! `credentials.json` shape:
//!
//! ```json
//! { "api_key": "...", "rest_endpoint": "https://rest.iad-01.braze.com" }
//! ```

use std::env;

use anyhow::{Context, Result};
use braze::{export::ExportUsersBySegmentRequest, Client};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let credentials_path = env::var("BRAZE_CREDENTIALS")
        .context("set BRAZE_CREDENTIALS to a JSON file with api_key + rest_endpoint")?;

    let segment_id = env::args()
        .nth(1)
        .context("pass the segment_id as the first CLI arg")?;

    let client = Client::builder()
        .credentials_file(&credentials_path)?
        .build()?;

    let response = client
        .export()
        .users_by_segment(&ExportUsersBySegmentRequest {
            segment_id,
            fields_to_export: vec!["external_id".into(), "email".into()],
            ..Default::default()
        })
        .send()
        .await?;

    println!("message: {:?}", response.message);
    println!("object_prefix: {:?}", response.object_prefix);
    println!("url: {:?}", response.url);

    Ok(())
}
