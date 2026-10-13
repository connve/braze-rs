//! Errors returned by the Braze SDK.

use reqwest::StatusCode;

/// All errors surfaced by the SDK.
///
/// Every public method that can fail returns this type. Use
/// [`Error::is_retryable`] to decide whether to retry without pattern-matching
/// the variants.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// No API key was provided when building the client.
    #[error("API key is required to call Braze")]
    MissingApiKey,

    /// The API key contains characters that can't be sent in an HTTP header.
    #[error("API key contains characters that are not allowed in an HTTP header")]
    InvalidApiKey,

    /// The REST endpoint URL could not be parsed.
    #[error("invalid Braze REST endpoint")]
    InvalidEndpoint {
        /// Underlying URL parse error.
        #[source]
        source: url::ParseError,
    },

    /// Reading credentials from disk failed.
    #[error("failed to read credentials file")]
    CredentialsIo {
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Parsing the credentials JSON failed.
    #[error("failed to parse credentials file")]
    CredentialsParse {
        /// Underlying JSON decode error.
        #[source]
        source: serde_json::Error,
    },

    /// A transport-layer failure (connection reset, TLS, DNS, timeout, etc.).
    #[error("HTTP transport error")]
    Http {
        /// Underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },

    /// The Braze API returned a non-success status.
    #[error("Braze API returned {status}: {body}")]
    Api {
        /// HTTP status code from the response.
        status: StatusCode,
        /// Raw response body so callers can surface Braze's structured
        /// error details.
        body: String,
    },

    /// The response body could not be deserialized into the expected type.
    #[error("failed to deserialize Braze response")]
    Deserialize {
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },

    /// A request header is either reserved by the SDK (see
    /// [`Builder::default_headers`](crate::Builder::default_headers)) or could
    /// not be converted into a valid `HeaderName` / `HeaderValue`.
    #[error("invalid request header `{name}`")]
    InvalidHeader {
        /// The header name that was rejected.
        name: String,
    },

    /// A caller-supplied ID used in the request path is empty, `.`, `..`, or
    /// contains control characters, so it can't be sent as a single path
    /// segment.
    #[error("invalid path segment `{segment}`")]
    InvalidPathSegment {
        /// The rejected value.
        segment: String,
    },
}

impl Error {
    /// Whether the operation is worth retrying.
    ///
    /// Returns `true` for transport failures, HTTP 408 / 425 / 429, and any
    /// 5xx response. Auth and other 4xx responses return `false`.
    ///
    /// This only says the failure may be transient. Braze can have processed
    /// the request before a timeout or 5xx reached you, and the messaging send
    /// endpoints have no idempotency key, so retrying one of them can deliver
    /// a message twice. Retry sends only if a duplicate is acceptable.
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
            | Error::InvalidEndpoint { .. }
            | Error::CredentialsIo { .. }
            | Error::CredentialsParse { .. }
            | Error::Deserialize { .. }
            | Error::InvalidHeader { .. }
            | Error::InvalidPathSegment { .. } => false,
        }
    }
}

/// Shorthand `Result` returned by SDK methods.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_classification() {
        let rate_limited = Error::Api {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: String::new(),
        };
        assert!(rate_limited.is_retryable());

        let server = Error::Api {
            status: StatusCode::BAD_GATEWAY,
            body: String::new(),
        };
        assert!(server.is_retryable());

        let unauthorized = Error::Api {
            status: StatusCode::UNAUTHORIZED,
            body: String::new(),
        };
        assert!(!unauthorized.is_retryable());

        assert!(!Error::MissingApiKey.is_retryable());
        assert!(!Error::InvalidHeader {
            name: String::new()
        }
        .is_retryable());
    }
}
