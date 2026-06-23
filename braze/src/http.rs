//! Shared HTTP plumbing used by every API module.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};

use crate::error::{Error, Result};

#[cfg(feature = "export")]
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
#[cfg(feature = "export")]
use serde::{de::DeserializeOwned, Serialize};
#[cfg(feature = "export")]
use url::Url;

pub(crate) const USER_AGENT: &str = concat!("braze-rs/", env!("CARGO_PKG_VERSION"));

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Build the shared `reqwest::Client` used by every API module.
///
/// All requests get `Accept: application/json` and `User-Agent: braze-rs/<ver>`.
/// The bearer token is injected per-request so a single client can serve
/// callers that rotate keys.
pub(crate) fn build_reqwest_client() -> Result<reqwest::Client> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(DEFAULT_TIMEOUT)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .default_headers(default_headers)
        .build()
        .map_err(|source| Error::Http { source })
}

#[cfg(feature = "export")]
pub(crate) fn bearer_header(api_key: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|_| Error::MissingApiKey)
}

/// POST a JSON body to `path` (relative to `base`), and deserialize the
/// response into `R`.
#[cfg(feature = "export")]
pub(crate) async fn post_json<B, R>(
    client: &reqwest::Client,
    base: &Url,
    api_key: &str,
    path: &str,
    body: &B,
) -> Result<R>
where
    B: Serialize + ?Sized,
    R: DeserializeOwned,
{
    let url = base
        .join(path)
        .map_err(|source| Error::InvalidEndpoint { source })?;

    let response = client
        .post(url)
        .header(AUTHORIZATION, bearer_header(api_key)?)
        .header(CONTENT_TYPE, "application/json")
        .json(body)
        .send()
        .await
        .map_err(|source| Error::Http { source })?;

    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|source| Error::Http { source })?;

    if !status.is_success() {
        return Err(Error::Api {
            status,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        });
    }

    serde_json::from_slice(&bytes).map_err(|source| Error::Deserialize { source })
}
