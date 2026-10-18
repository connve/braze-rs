//! Errors returned by the Braze SDK.

use std::time::Duration;

use reqwest::StatusCode;

/// All errors surfaced by the SDK.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No API key was provided when building the client.
    #[error("API key is required to call Braze")]
    MissingApiKey,

    /// The API key contains characters that can't be sent in an HTTP header.
    #[error("API key contains characters that are not allowed in an HTTP header")]
    InvalidApiKey,

    /// No base URL was provided when building the client.
    #[error("base URL is required to call Braze")]
    MissingBaseUrl,

    /// The base URL could not be parsed, or can't carry a path.
    #[error("invalid Braze base URL")]
    InvalidBaseUrl {
        /// Underlying URL parse error.
        #[source]
        source: url::ParseError,
    },

    /// A transport-layer failure (connection reset, TLS, DNS, timeout, etc.).
    #[error("HTTP transport error")]
    Http {
        /// Underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },

    /// The Braze API returned a non-success status. The message leaves out
    /// `body`, which can contain personal data.
    #[error("Braze API returned {status}")]
    Api {
        /// HTTP status code from the response.
        status: StatusCode,
        /// How long Braze asked to wait before retrying a 429, if it said.
        retry_after: Option<Duration>,
        /// Raw response body.
        body: String,
    },

    /// The response body could not be deserialized into the expected type.
    #[error("failed to deserialize Braze response")]
    Deserialize {
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// A request header is invalid, or reserved by the SDK (see
    /// [`Builder::default_headers`](crate::Builder::default_headers)).
    #[error("invalid request header `{name}`")]
    InvalidHeader {
        /// The header name that was rejected.
        name: String,
    },

    /// An ID used in the request path is empty, `.`, `..`, or contains
    /// control characters.
    #[error("invalid path segment `{segment}`")]
    InvalidPathSegment {
        /// The rejected value.
        segment: String,
    },
}

impl Error {
    /// `true` for transport failures, HTTP 408 / 425 / 429, and 5xx.
    ///
    /// Braze may have processed the request before a timeout or 5xx.
    /// Messaging sends have no idempotency key, so retrying one can deliver
    /// a message twice.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Http { source } => {
                source.is_timeout() || source.is_connect() || source.is_request()
            }
            Error::Api { status, .. } => {
                status.is_server_error()
                    || matches!(
                        *status,
                        StatusCode::REQUEST_TIMEOUT
                            | StatusCode::TOO_EARLY
                            | StatusCode::TOO_MANY_REQUESTS
                    )
            }
            Error::MissingApiKey
            | Error::InvalidApiKey
            | Error::MissingBaseUrl
            | Error::InvalidBaseUrl { .. }
            | Error::Deserialize { .. }
            | Error::InvalidHeader { .. }
            | Error::InvalidPathSegment { .. } => false,
        }
    }

    /// How long to wait before retrying a rate-limited (429) request. Prefer
    /// it over your own backoff.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

/// Shorthand `Result` returned by SDK methods.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    fn api_error(status: StatusCode) -> Error {
        Error::Api {
            status,
            retry_after: None,
            body: String::new(),
        }
    }

    #[test]
    fn retryable_classification() {
        assert!(api_error(StatusCode::TOO_MANY_REQUESTS).is_retryable());
        assert!(api_error(StatusCode::BAD_GATEWAY).is_retryable());
        assert!(!api_error(StatusCode::UNAUTHORIZED).is_retryable());
        assert!(!Error::MissingApiKey.is_retryable());
        assert!(!Error::InvalidHeader {
            name: String::new()
        }
        .is_retryable());
    }

    #[test]
    fn retry_after_is_only_reported_for_api_errors() {
        let throttled = Error::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            retry_after: Some(Duration::from_secs(7)),
            body: String::new(),
        };
        assert_eq!(throttled.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(Error::MissingApiKey.retry_after(), None);
    }

    #[test]
    fn api_error_message_leaves_out_body() {
        let error = Error::Api {
            status: StatusCode::BAD_REQUEST,
            retry_after: None,
            body: "ada@example.com".into(),
        };
        let message = error.to_string();
        assert!(!message.contains("ada@example.com"), "{message}");
    }
}
