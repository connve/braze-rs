//! Integration tests for the export endpoints.
//!
//! Uses only unknown IDs, so no export is started.
//!
//! Runs against the Braze test workspace in `BRAZE_API_KEY` /
//! `BRAZE_BASE_URL`, so every test is `#[ignore]`d; run them with
//! `cargo test -- --ignored`.

mod common;

use braze::export::{ExportUsersByIdsRequest, ExportUsersBySegmentRequest};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn export_users_by_unknown_segment_is_rejected() -> Result {
    let client = common::client()?;

    let result = client
        .export()
        .users_by_segment(&ExportUsersBySegmentRequest {
            segment_id: unique_name!("segment"),
            fields_to_export: vec!["external_id".into()],
            ..Default::default()
        })
        .send()
        .await;

    ensure_rejected!(result);
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn export_users_by_unknown_external_id_returns_invalid() -> Result {
    let client = common::client()?;
    let unknown_id = unique_name!("user");

    let response = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest {
            external_ids: vec![unknown_id.clone()],
            fields_to_export: vec!["external_id".into(), "email".into()],
            ..Default::default()
        })
        .send()
        .await?;

    ensure!(
        response.users.is_empty(),
        "expected no users, got {}",
        response.users.len()
    );
    ensure!(
        response.invalid_user_ids.contains(&unknown_id),
        "expected {unknown_id} in invalid_user_ids, got {:?}",
        response.invalid_user_ids
    );
    Ok(())
}

#[tokio::test]
#[ignore = "needs a Braze test workspace in BRAZE_API_KEY and BRAZE_BASE_URL"]
async fn export_users_requires_at_least_one_identifier() -> Result {
    let client = common::client()?;

    let result = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest::default())
        .send()
        .await;

    ensure!(
        matches!(&result, Err(braze::Error::Api { status, .. }) if status.as_u16() == 400),
        "expected a 400, got: {result:?}"
    );
    Ok(())
}
