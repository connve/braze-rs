//! Run with:
//!
//! ```bash
//! export BRAZE_CREDENTIALS=$PWD/credentials.json
//! cargo run --example export_users -- user_id_1 user_id_2
//! ```
//!
//! `credentials.json` shape:
//!
//! ```json
//! { "api_key": "...", "rest_endpoint": "https://rest.iad-01.braze.com" }
//! ```

use std::env;

use anyhow::{Context, Result};
use braze::{export::ExportUsersByIdsRequest, Client};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let credentials_path = env::var("BRAZE_CREDENTIALS")
        .context("set BRAZE_CREDENTIALS to a JSON file with api_key + rest_endpoint")?;

    let external_ids: Vec<String> = env::args().skip(1).collect();
    if external_ids.is_empty() {
        anyhow::bail!("pass at least one external_id as a CLI arg");
    }

    let client = Client::builder()
        .credentials_file(&credentials_path)?
        .build()?;

    let response = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest {
            external_ids: Some(external_ids),
            fields_to_export: Some(vec![
                "external_id".into(),
                "email".into(),
                "first_name".into(),
                "last_name".into(),
            ]),
            ..Default::default()
        })
        .await?;

    println!("message: {}", response.message);
    println!("users returned: {}", response.users.len());
    println!("invalid ids: {:?}", response.invalid_user_ids);

    for user in &response.users {
        println!("  - {:?} ({:?})", user.external_id, user.email);
    }

    Ok(())
}
