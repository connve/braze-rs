//! Braze client, credentials, and builder.

use std::path::Path;
#[cfg(feature = "_api")]
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

use crate::error::{Error, Result};
use crate::headers;

/// Authentication material for the Braze REST API.
///
/// `rest_endpoint` is the per-instance base URL (for example
/// `https://rest.iad-01.braze.com`). The Braze dashboard surfaces this value
/// under *Settings → API Keys*.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// REST API key with the permissions required by the calls you'll make.
    ///
    /// Redacted from `Debug` output and zeroed on drop.
    pub api_key: SecretString,
    /// Per-instance REST endpoint, e.g. `https://rest.iad-01.braze.com`.
    pub rest_endpoint: String,
}

impl Credentials {
    /// Load credentials from a JSON file.
    ///
    /// ```json
    /// { "api_key": "...", "rest_endpoint": "https://rest.iad-01.braze.com" }
    /// ```
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).map_err(|source| Error::CredentialsIo { source })?;
        serde_json::from_str(&contents).map_err(|source| Error::CredentialsParse { source })
    }
}

#[cfg(feature = "_api")]
#[derive(Debug)]
struct Inner {
    /// `Bearer <api key>`, marked sensitive so `Debug` output redacts it.
    authorization: HeaderValue,
    base_url: Url,
    default_headers: HeaderMap,
    http: reqwest::Client,
}

/// Thread-safe Braze API client.
///
/// `Client` is cheap to clone — internal state lives behind an `Arc`. Use a
/// single instance for the lifetime of your process.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Client {
    #[cfg(feature = "_api")]
    inner: Arc<Inner>,
}

impl Client {
    /// Start a [`Builder`].
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Access the `/users/export/*` endpoints.
    #[cfg(feature = "export")]
    pub fn export(&self) -> crate::export::Export<'_> {
        crate::export::Export::new(self)
    }

    /// Access the messaging endpoints.
    #[cfg(feature = "messaging")]
    pub fn messaging(&self) -> crate::messaging::Messaging<'_> {
        crate::messaging::Messaging::new(self)
    }

    /// With no API feature enabled there is nothing to send requests with,
    /// so the validated parts are dropped.
    #[cfg(not(feature = "_api"))]
    fn new(
        _authorization: HeaderValue,
        _base_url: Url,
        _default_headers: HeaderMap,
    ) -> Result<Self> {
        Ok(Self {})
    }
}

#[cfg(feature = "_api")]
impl Client {
    fn new(authorization: HeaderValue, base_url: Url, default_headers: HeaderMap) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Inner {
                authorization,
                base_url,
                default_headers,
                http: crate::http::build_reqwest_client()?,
            }),
        })
    }

    pub(crate) fn authorization(&self) -> &HeaderValue {
        &self.inner.authorization
    }

    pub(crate) fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    pub(crate) fn default_headers(&self) -> &HeaderMap {
        &self.inner.default_headers
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }
}

/// Builds `Bearer <api key>` once, marked sensitive so `Debug` output of the
/// value and of reqwest's request redacts it.
fn authorization_header(api_key: &str) -> Result<HeaderValue> {
    let bearer = SecretString::from(format!("Bearer {api_key}"));
    let mut value =
        HeaderValue::from_str(bearer.expose_secret()).map_err(|_| Error::InvalidApiKey)?;
    value.set_sensitive(true);
    Ok(value)
}

/// Endpoint paths are appended as segments, so the URL must be able to carry
/// a path (`https://...`, not `mailto:...`).
fn parse_base_url(rest_endpoint: &str) -> Result<Url> {
    let base_url = Url::parse(rest_endpoint).map_err(|source| Error::InvalidEndpoint { source })?;
    if base_url.cannot_be_a_base() {
        return Err(Error::InvalidEndpoint {
            source: url::ParseError::RelativeUrlWithCannotBeABaseBase,
        });
    }
    Ok(base_url)
}

/// Fluent constructor for [`Client`].
#[derive(Debug, Default)]
pub struct Builder {
    credentials: Option<Credentials>,
    default_headers: HeaderMap,
}

impl Builder {
    /// Headers sent with every request made by the built client.
    ///
    /// Per-call headers set with `.header()` / `.headers()` on a request
    /// builder take precedence over a default with the same name. Headers
    /// reserved by the SDK (`Authorization`, `Content-Type`, `Content-Length`,
    /// `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`) make
    /// [`Builder::build`] fail with [`Error::InvalidHeader`].
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }

    /// Provide credentials directly.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Load credentials from a JSON file at `path`.
    pub fn credentials_file(mut self, path: impl AsRef<Path>) -> Result<Self> {
        self.credentials = Some(Credentials::from_file(path)?);
        Ok(self)
    }

    /// Build the client. Fails if no credentials were supplied, the API key
    /// is blank or not valid in an HTTP header, `rest_endpoint` cannot be
    /// parsed as a URL, or a default header is reserved by the SDK.
    ///
    /// Surrounding whitespace in the API key (such as a trailing newline from
    /// a credentials file) is ignored.
    pub fn build(self) -> Result<Client> {
        headers::reject_forbidden(&self.default_headers)?;

        let credentials = self.credentials.ok_or(Error::MissingApiKey)?;
        let api_key = credentials.api_key.expose_secret().trim();
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        let authorization = authorization_header(api_key)?;
        let base_url = parse_base_url(&credentials.rest_endpoint)?;

        Client::new(authorization, base_url, self.default_headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_rejects_empty_api_key() {
        let result = Client::builder()
            .credentials(Credentials {
                api_key: "   ".into(),
                rest_endpoint: "https://rest.iad-01.braze.com".into(),
            })
            .build();
        assert!(matches!(result, Err(Error::MissingApiKey)));
    }

    #[test]
    fn builder_rejects_missing_credentials() {
        assert!(matches!(
            Client::builder().build(),
            Err(Error::MissingApiKey)
        ));
    }

    #[test]
    fn builder_rejects_invalid_endpoint() {
        let result = Client::builder()
            .credentials(Credentials {
                api_key: "k".into(),
                rest_endpoint: "not a url".into(),
            })
            .build();
        assert!(matches!(result, Err(Error::InvalidEndpoint { .. })));
    }

    #[test]
    fn builder_rejects_endpoint_without_path() {
        let result = Client::builder()
            .credentials(Credentials {
                api_key: "k".into(),
                rest_endpoint: "mailto:ops@example.com".into(),
            })
            .build();
        assert!(matches!(result, Err(Error::InvalidEndpoint { .. })));
    }

    #[test]
    fn builder_rejects_api_key_not_valid_in_a_header() {
        let result = Client::builder()
            .credentials(Credentials {
                api_key: "abc\ndef".into(),
                rest_endpoint: "https://rest.iad-01.braze.com".into(),
            })
            .build();
        assert!(matches!(result, Err(Error::InvalidApiKey)));
    }

    #[cfg(feature = "_api")]
    #[test]
    fn builder_trims_api_key() {
        let client = Client::builder()
            .credentials(Credentials {
                api_key: "key-from-file\n".into(),
                rest_endpoint: "https://rest.iad-01.braze.com".into(),
            })
            .build()
            .expect("client builds");
        assert_eq!(client.authorization(), "Bearer key-from-file");
    }

    #[cfg(feature = "_api")]
    #[test]
    fn debug_output_does_not_contain_api_key() {
        let credentials = Credentials {
            api_key: "super-secret-key".into(),
            rest_endpoint: "https://rest.iad-01.braze.com".into(),
        };
        let builder = Client::builder().credentials(credentials.clone());
        let client = Client::builder()
            .credentials(credentials.clone())
            .build()
            .expect("client builds");

        for output in [
            format!("{credentials:?}"),
            format!("{builder:?}"),
            format!("{client:?}"),
        ] {
            assert!(!output.contains("super-secret-key"), "leaked: {output}");
        }
    }

    #[test]
    fn builder_rejects_reserved_default_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_static("Bearer other"),
        );
        let result = Client::builder()
            .credentials(Credentials {
                api_key: "k".into(),
                rest_endpoint: "https://rest.iad-01.braze.com".into(),
            })
            .default_headers(headers)
            .build();
        assert!(matches!(
            result,
            Err(Error::InvalidHeader { ref name }) if name == "authorization"
        ));
    }
}
