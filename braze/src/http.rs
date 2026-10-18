//! Shared HTTP plumbing used by every API module.

use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use crate::client::Client;
use crate::error::{Error, Result};

pub(crate) const USER_AGENT: &str = concat!("braze-rs/", env!("CARGO_PKG_VERSION"));

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Caller default headers are merged per request in [`post_json`], not set
/// here: reqwest keeps only the last value of a multi-valued default header.
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

/// Each segment is percent-encoded, so an ID can't escape it. `url` would
/// silently drop `.` / `..` and strip control characters, so those are
/// rejected instead.
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
        .map_err(|()| Error::InvalidBaseUrl {
            source: url::ParseError::RelativeUrlWithCannotBeABaseBase,
        })?
        .pop_if_empty()
        .extend(segments);
    Ok(url)
}

/// `headers` are the per-call headers, already checked by
/// `HeaderBag::finish`.
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
    let retry_after = match status {
        StatusCode::TOO_MANY_REQUESTS => retry_after(response.headers(), Utc::now()),
        _ => None,
    };
    let bytes = response
        .bytes()
        .await
        .map_err(|source| Error::Http { source })?;

    if !status.is_success() {
        return Err(Error::Api {
            status,
            retry_after,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        });
    }

    serde_json::from_slice(&bytes).map_err(|source| Error::Deserialize { source })
}

/// Braze sends `X-RateLimit-Retry-After` in seconds when it knows the delay,
/// and `X-RateLimit-Reset`, the end of the rate-limit window in UTC epoch
/// seconds, on every response.
fn retry_after(headers: &HeaderMap, now: DateTime<Utc>) -> Option<Duration> {
    let header = |name: &str| headers.get(name)?.to_str().ok()?.trim().parse::<f64>().ok();

    let seconds = match header("x-ratelimit-retry-after") {
        Some(seconds) => seconds,
        None => header("x-ratelimit-reset")? - now.timestamp() as f64,
    };
    Duration::try_from_secs_f64(seconds.max(0.0)).ok()
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

    fn rate_limit_headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, HeaderValue::from_static(value));
        }
        headers
    }

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("valid timestamp")
    }

    #[test]
    fn retry_after_prefers_the_explicit_delay() {
        let headers = rate_limit_headers(&[
            ("x-ratelimit-retry-after", "12"),
            ("x-ratelimit-reset", "1700000060"),
        ]);
        assert_eq!(retry_after(&headers, now()), Some(Duration::from_secs(12)));
    }

    #[test]
    fn retry_after_accepts_fractional_seconds() {
        let headers = rate_limit_headers(&[("x-ratelimit-retry-after", "1.5")]);
        assert_eq!(
            retry_after(&headers, now()),
            Some(Duration::from_millis(1500))
        );
    }

    #[test]
    fn retry_after_falls_back_to_the_window_reset() {
        let headers = rate_limit_headers(&[("x-ratelimit-reset", "1700000045")]);
        assert_eq!(retry_after(&headers, now()), Some(Duration::from_secs(45)));
    }

    #[test]
    fn retry_after_is_zero_for_a_reset_in_the_past() {
        let headers = rate_limit_headers(&[("x-ratelimit-reset", "1699999990")]);
        assert_eq!(retry_after(&headers, now()), Some(Duration::ZERO));
    }

    #[test]
    fn retry_after_is_none_without_rate_limit_headers() {
        let headers = rate_limit_headers(&[("x-ratelimit-reset", "soon")]);
        assert_eq!(retry_after(&headers, now()), None);
        assert_eq!(retry_after(&HeaderMap::new(), now()), None);
    }
}
