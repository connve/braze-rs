//! Braze client, credentials, and builder.

#[cfg(feature = "_api")]
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

use crate::error::{Error, Result};
use crate::headers;

/// Authentication for one Braze workspace.
///
/// Deserializes from `{ "api_key": "..." }`, ignoring other fields, so an
/// application can load it from wherever it keeps secrets.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// REST API key from *Settings → APIs and Identifiers*, with the
    /// permissions the calls need. Redacted from `Debug` output and zeroed on
    /// drop.
    pub api_key: SecretString,
}

#[cfg(feature = "_api")]
#[derive(Debug)]
struct Inner {
    authorization: HeaderValue,
    base_url: Url,
    default_headers: HeaderMap,
    http: reqwest::Client,
}

/// Braze API client for one workspace. Cheap to clone.
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

    /// Export endpoints (`/users/export/*`).
    #[cfg(feature = "export")]
    pub fn export(&self) -> crate::export::Export<'_> {
        crate::export::Export::new(self)
    }

    /// Messaging endpoints.
    #[cfg(feature = "messaging")]
    pub fn messaging(&self) -> crate::messaging::Messaging<'_> {
        crate::messaging::Messaging::new(self)
    }

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

/// Marked sensitive, so `Debug` output of it and of reqwest requests redacts
/// the key.
fn authorization_header(api_key: &str) -> Result<HeaderValue> {
    let value = SecretString::from(format!("Bearer {api_key}"));
    let mut value =
        HeaderValue::from_str(value.expose_secret()).map_err(|_| Error::InvalidApiKey)?;
    value.set_sensitive(true);
    Ok(value)
}

/// Paths are appended as segments, so the URL must be able to carry one.
fn parse_base_url(base_url: &str) -> Result<Url> {
    let url = Url::parse(base_url).map_err(|source| Error::InvalidBaseUrl { source })?;
    if url.cannot_be_a_base() {
        return Err(Error::InvalidBaseUrl {
            source: url::ParseError::RelativeUrlWithCannotBeABaseBase,
        });
    }
    Ok(url)
}

/// Fluent constructor for [`Client`].
#[derive(Debug, Default)]
pub struct Builder {
    credentials: Option<Credentials>,
    base_url: Option<String>,
    default_headers: HeaderMap,
}

impl Builder {
    /// Headers sent with every request. Per-call headers of the same name
    /// win. Reserved headers (`Authorization`, `Content-Type`,
    /// `Content-Length`, `Transfer-Encoding`, `Accept`, `Host`, `User-Agent`)
    /// make [`Builder::build`] fail with [`Error::InvalidHeader`].
    pub fn default_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }

    /// The workspace's REST API key.
    pub fn api_key(self, api_key: impl Into<SecretString>) -> Self {
        self.credentials(Credentials {
            api_key: api_key.into(),
        })
    }

    /// Credentials loaded by the application.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// The REST endpoint of the workspace's Braze instance, such as
    /// `https://rest.iad-01.braze.com`, from *Settings → APIs and
    /// Identifiers*. Required: each instance has its own host.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Build the client. Whitespace around the API key is ignored.
    pub fn build(self) -> Result<Client> {
        headers::reject_forbidden(&self.default_headers)?;

        let credentials = self.credentials.ok_or(Error::MissingApiKey)?;
        let api_key = credentials.api_key.expose_secret().trim();
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        let authorization = authorization_header(api_key)?;
        let base_url = parse_base_url(self.base_url.as_deref().ok_or(Error::MissingBaseUrl)?)?;

        Client::new(authorization, base_url, self.default_headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_URL: &str = "https://rest.iad-01.braze.com";

    #[test]
    fn builder_rejects_empty_api_key() {
        let result = Client::builder().api_key("   ").base_url(BASE_URL).build();
        assert!(matches!(result, Err(Error::MissingApiKey)));
    }

    #[test]
    fn builder_rejects_missing_api_key() {
        let result = Client::builder().base_url(BASE_URL).build();
        assert!(matches!(result, Err(Error::MissingApiKey)));
    }

    #[test]
    fn builder_rejects_missing_base_url() {
        let result = Client::builder().api_key("k").build();
        assert!(matches!(result, Err(Error::MissingBaseUrl)));
    }

    #[test]
    fn builder_rejects_invalid_base_url() {
        let result = Client::builder().api_key("k").base_url("not a url").build();
        assert!(matches!(result, Err(Error::InvalidBaseUrl { .. })));
    }

    #[test]
    fn builder_rejects_base_url_without_path() {
        let result = Client::builder()
            .api_key("k")
            .base_url("mailto:ops@example.com")
            .build();
        assert!(matches!(result, Err(Error::InvalidBaseUrl { .. })));
    }

    #[test]
    fn builder_rejects_api_key_not_valid_in_a_header() {
        let result = Client::builder()
            .api_key("abc\ndef")
            .base_url(BASE_URL)
            .build();
        assert!(matches!(result, Err(Error::InvalidApiKey)));
    }

    #[test]
    fn credentials_ignore_extra_fields() {
        let parsed: Credentials =
            serde_json::from_str(r#"{ "api_key": "k", "base_url": "https://example.com" }"#)
                .expect("credentials parse");
        assert_eq!(parsed.api_key.expose_secret(), "k");
    }

    #[cfg(feature = "_api")]
    #[test]
    fn builder_trims_api_key() {
        let client = Client::builder()
            .api_key("key\n")
            .base_url(BASE_URL)
            .build()
            .expect("client builds");
        assert_eq!(client.authorization(), "Bearer key");
    }

    #[cfg(feature = "_api")]
    #[test]
    fn debug_output_does_not_contain_api_key() {
        let credentials = Credentials {
            api_key: "super-secret-key".into(),
        };
        let builder = Client::builder().credentials(credentials.clone());
        let client = Client::builder()
            .credentials(credentials.clone())
            .base_url(BASE_URL)
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
            .api_key("k")
            .base_url(BASE_URL)
            .default_headers(headers)
            .build();
        assert!(matches!(
            result,
            Err(Error::InvalidHeader { ref name }) if name == "authorization"
        ));
    }
}
