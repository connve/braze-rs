//! Run with:
//!
//! ```bash
//! export BRAZE_API_KEY=...
//! export BRAZE_BASE_URL=https://rest.iad-01.braze.com
//! cargo run --example export_segment --features export -- <segment_id>
//! ```
//!
//! Starts an export of every user in the segment. Files go to your configured
//! storage bucket, or to the temporary `url` printed below.

use std::env;

use anyhow::{Context, Result};
use braze::{export::ExportUsersBySegmentRequest, Client};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = env::var("BRAZE_API_KEY").context("set BRAZE_API_KEY to a REST API key")?;
    let base_url =
        env::var("BRAZE_BASE_URL").context("set BRAZE_BASE_URL to the instance's REST endpoint")?;

    let segment_id = env::args()
        .nth(1)
        .context("pass the segment_id as the first CLI arg")?;

    let client = Client::builder()
        .api_key(api_key)
        .base_url(base_url)
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
