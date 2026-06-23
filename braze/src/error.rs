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
}

impl Error {
    /// Whether the operation is worth retrying.
    ///
    /// Returns `true` for transport failures, HTTP 408 / 425 / 429, and any
    /// 5xx response. Auth and other 4xx responses return `false`.
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
            | Error::InvalidEndpoint { .. }
            | Error::CredentialsIo { .. }
            | Error::CredentialsParse { .. }
            | Error::Deserialize { .. } => false,
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
    }
}
