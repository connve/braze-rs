//! Helpers for live tests against the test workspace whose REST API key is in
//! `BRAZE_API_KEY` and whose REST endpoint is in `BRAZE_BASE_URL`.
//!
//! The checks return an error instead of panicking. They are macros so files
//! that use only some of them don't hit `dead_code`.

use braze::Client;
use std::env;

/// The test workspace's REST API key.
const API_KEY_ENV: &str = "BRAZE_API_KEY";

/// The test workspace's REST endpoint, e.g. `https://rest.iad-01.braze.com`.
const BASE_URL_ENV: &str = "BRAZE_BASE_URL";

/// A client for the test workspace. Fails when either variable is unset, so
/// an ignored test run without them can't pass.
pub fn client() -> Result<Client, Box<dyn std::error::Error>> {
    let api_key = env::var(API_KEY_ENV).map_err(|error| format!("{API_KEY_ENV}: {error}"))?;
    let base_url = env::var(BASE_URL_ENV).map_err(|error| format!("{BASE_URL_ENV}: {error}"))?;
    let client = Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()?;
    Ok(client)
}

/// `assert!` that returns an error instead of panicking.
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $($message:tt)+) => {
        if !$condition {
            return Err(format!($($message)+).into());
        }
    };
}

/// Expects a non-retryable 4xx other than 401/403: the key worked and Braze
/// rejected the request itself. A 401/403 means auth is broken or the key
/// lacks the endpoint's permission, which the live tests exist to catch.
#[macro_export]
macro_rules! ensure_rejected {
    ($result:expr) => {
        match $result {
            Err(error) => {
                let rejected = matches!(
                    &error,
                    braze::Error::Api { status, .. }
                        if status.is_client_error()
                            && status.as_u16() != 401
                            && status.as_u16() != 403
                );
                if !rejected || error.is_retryable() {
                    return Err(format!(
                        "expected a non-retryable 4xx other than 401/403, got: {error:?}"
                    )
                    .into());
                }
            }
            Ok(value) => {
                return Err(format!("expected a rejection, got: {value:?}").into());
            }
        }
    };
}

/// A name no other run uses, for IDs that must not exist in the workspace.
#[macro_export]
macro_rules! unique_name {
    ($prefix:expr) => {
        format!(
            "braze-rs-test-{}-{}",
            $prefix,
            ::braze::chrono::Utc::now().format("%Y%m%d%H%M%S%9f")
        )
    };
}
