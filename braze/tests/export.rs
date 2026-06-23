//! Integration tests for the Export API.

mod common;

use braze::export::ExportUsersByIdsRequest;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn export_users_by_unknown_external_id_returns_invalid() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let unknown_id = format!("braze-rs-test-{}", uuid_like());

    let response = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest {
            external_ids: Some(vec![unknown_id.clone()]),
            fields_to_export: Some(vec!["external_id".into(), "email".into()]),
            ..Default::default()
        })
        .await?;

    assert_eq!(response.message, "success");
    assert!(response.users.is_empty());
    assert!(response.invalid_user_ids.contains(&unknown_id));

    Ok(())
}

#[tokio::test]
async fn export_users_requires_at_least_one_identifier() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let result = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest::default())
        .await;

    // Braze returns 400 when no identifier is supplied.
    let err = result.expect_err("call without identifiers should fail");
    assert!(
        matches!(err, braze::Error::Api { status, .. } if status.as_u16() == 400),
        "expected 400 Api error, got: {err:?}"
    );

    Ok(())
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}
