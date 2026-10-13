//! Shared HTTP plumbing used by every API module.

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use crate::client::Client;
use crate::error::{Error, Result};

pub(crate) const USER_AGENT: &str = concat!("braze-rs/", env!("CARGO_PKG_VERSION"));

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Build the shared `reqwest::Client` used by every API module.
///
/// All requests get `Accept: application/json` and `User-Agent: braze-rs/<ver>`.
/// Caller-supplied default headers are merged per request in [`post_json`]
/// rather than set here, because reqwest keeps only the last value of a
/// multi-valued default header.
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

/// Append `segments` to `base`, percent-encoding each one so caller-supplied
/// IDs cannot escape their path segment.
///
/// `url` drops `.` and `..` segments and strips tabs and newlines before it
/// checks for them, so those are rejected here instead of being sent as a
/// different path.
pub(crate) fn endpoint_url(base: &Url, segments: &[&str]) -> Result<Url> {
    if let Some(segment) = segments.iter().find(|segment| {
        segment.is_empty()
            || **segment == "."
            || **segment == ".."
            || segment.contains(char::is_control)
    }) {
        return Err(Error::InvalidPathSegment {
            segment: (*segment).to_owned(),
        });
    }

    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|()| Error::InvalidEndpoint {
            source: url::ParseError::RelativeUrlWithCannotBeABaseBase,
        })?
        .pop_if_empty()
        .extend(segments);
    Ok(url)
}

/// POST a JSON body to the path built from `segments` (relative to the
/// client's base URL), and deserialize the response into `R`.
///
/// `headers` are the per-call headers from the request builder, already
/// checked against the reserved set by `HeaderBag::finish`. They override the
/// client's default headers with the same name.
pub(crate) async fn post_json<B, R>(
    client: &Client,
    segments: &[&str],
    headers: HeaderMap,
    body: &B,
) -> Result<R>
where
    B: Serialize + ?Sized,
    R: DeserializeOwned,
{
    let url = endpoint_url(client.base_url(), segments)?;
    let headers = crate::headers::merged(client.default_headers(), headers);

    let response = client
        .http()
        .post(url)
        .headers(headers)
        .header(AUTHORIZATION, client.authorization().clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_url_keeps_base_prefix() {
        let base = Url::parse("https://example.com/proxy/").expect("valid url");
        let url = endpoint_url(&base, &["messages", "send"]).expect("joins");
        assert_eq!(url.path(), "/proxy/messages/send");
    }

    #[test]
    fn endpoint_url_appends_to_base_without_trailing_slash() {
        let base = Url::parse("https://example.com/proxy").expect("valid url");
        let url = endpoint_url(&base, &["messages", "send"]).expect("joins");
        assert_eq!(url.path(), "/proxy/messages/send");
    }

    #[test]
    fn endpoint_url_encodes_caller_supplied_segments() {
        let base = Url::parse("https://example.com/").expect("valid url");
        let url = endpoint_url(&base, &["campaigns", "a/../b?x", "send"]).expect("joins");
        assert_eq!(url.path(), "/campaigns/a%2F..%2Fb%3Fx/send");
        assert!(url.query().is_none());

        let url = endpoint_url(&base, &["campaigns", "%2e%2e", "send"]).expect("joins");
        assert_eq!(url.path(), "/campaigns/%252e%252e/send");
    }

    #[test]
    fn endpoint_url_rejects_segments_url_would_rewrite() {
        let base = Url::parse("https://example.com/").expect("valid url");
        for segment in ["", ".", "..", "..\t", "a\nb"] {
            assert!(
                matches!(
                    endpoint_url(&base, &["campaigns", segment, "send"]),
                    Err(Error::InvalidPathSegment { segment: ref s }) if s == segment
                ),
                "{segment:?} should be rejected"
            );
        }
    }
}
