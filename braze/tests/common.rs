//! Shared test utilities for integration tests.
//!
//! Provides a credentials helper and a skip macro for tests that require a
//! live Braze account, addressed by `BRAZE_CREDENTIALS` (JSON file path).

use braze::Client;
use std::env;

/// Environment variable pointing to the credentials JSON file.
pub const CREDENTIALS_ENV: &str = "BRAZE_CREDENTIALS";

/// Returns `true` if the credentials environment variable is set.
pub fn credentials_available() -> bool {
    env::var(CREDENTIALS_ENV).is_ok()
}

/// Skips the current integration test when credentials are not configured.
///
/// Must be called at the start of every integration test. The test function
/// must return `Result<(), Box<dyn std::error::Error>>`.
#[macro_export]
macro_rules! skip_if_no_credentials {
    () => {
        if !common::credentials_available() {
            eprintln!(
                "Skipping integration test: set BRAZE_CREDENTIALS to a JSON file path to run."
            );
            return Ok(());
        }
    };
}

/// Builds a `Client` from the credentials JSON pointed to by
/// `BRAZE_CREDENTIALS`.
///
/// The JSON file should contain:
/// ```json
/// {
///   "api_key": "...",
///   "rest_endpoint": "https://rest.iad-01.braze.com"
/// }
/// ```
pub fn client() -> Result<Client, Box<dyn std::error::Error>> {
    let path = env::var(CREDENTIALS_ENV)?;
    let client = Client::builder().credentials_file(path)?.build()?;
    Ok(client)
}
