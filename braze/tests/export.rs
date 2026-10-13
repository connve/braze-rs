//! Integration tests for the Export API.

mod common;

use braze::export::{ExportUsersByIdsRequest, ExportUsersBySegmentRequest};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Uses a segment ID that doesn't exist, so no export is started.
#[tokio::test]
async fn export_users_by_unknown_segment_is_rejected() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let result = client
        .export()
        .users_by_segment(&ExportUsersBySegmentRequest {
            segment_id: common::unknown_id("segment"),
            fields_to_export: vec!["external_id".into()],
            ..Default::default()
        })
        .send()
        .await;

    common::assert_rejected_by_braze(result);
    Ok(())
}

#[tokio::test]
async fn export_users_by_unknown_external_id_returns_invalid() -> Result {
    skip_if_no_credentials!();

    let client = common::client()?;

    let unknown_id = common::unknown_id("user");

    let response = client
        .export()
        .users_by_ids(&ExportUsersByIdsRequest {
            external_ids: vec![unknown_id.clone()],
            fields_to_export: vec!["external_id".into(), "email".into()],
            ..Default::default()
        })
        .send()
        .await?;

    assert_eq!(response.message.as_deref(), Some("success"));
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
        .send()
        .await;

    // Braze returns 400 when no identifier is supplied.
    let err = result.expect_err("call without identifiers should fail");
    assert!(
        matches!(err, braze::Error::Api { status, .. } if status.as_u16() == 400),
        "expected 400 Api error, got: {err:?}"
    );

    Ok(())
}
