//! Run with:
//!
//! ```bash
//! export BRAZE_API_KEY=...
//! export BRAZE_BASE_URL=https://rest.iad-01.braze.com
//! cargo run --example export_users --features export -- user_id_1 user_id_2
//! ```

use std::env;

use anyhow::{Context, Result};
use braze::{export::ExportUsersByIdsRequest, Client};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = env::var("BRAZE_API_KEY").context("set BRAZE_API_KEY to a REST API key")?;
    let base_url =
        env::var("BRAZE_BASE_URL").context("set BRAZE_BASE_URL to the instance's REST endpoint")?;

    let external_ids: Vec<String> = env::args().skip(1).collect();
    if external_ids.is_empty() {
        anyhow::bail!("pass at least one external_id as a CLI arg");
    }

    let client = Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()?;

    let response = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest {
            external_ids,
            fields_to_export: vec![
                "external_id".into(),
                "email".into(),
                "first_name".into(),
                "last_name".into(),
            ],
            ..Default::default()
        })
        .send()
        .await?;

    println!("message: {:?}", response.message);
    println!("users returned: {}", response.users.len());
    println!("invalid ids: {:?}", response.invalid_user_ids);

    for user in &response.users {
        println!("  - {:?} ({:?})", user.external_id, user.email);
    }

    Ok(())
}
