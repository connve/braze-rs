//! Braze client, credentials, and builder.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{Error, Result};
use crate::http;

/// Authentication material for the Braze REST API.
///
/// `rest_endpoint` is the per-instance base URL (for example
/// `https://rest.iad-01.braze.com`). The Braze dashboard surfaces this value
/// under *Settings → API Keys*.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// REST API key with the permissions required by the calls you'll make.
    pub api_key: String,
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

#[derive(Debug)]
struct Inner {
    api_key: String,
    base_url: Url,
    http: reqwest::Client,
}

/// Thread-safe Braze API client.
///
/// `Client` is cheap to clone — internal state lives behind an `Arc`. Use a
/// single instance for the lifetime of your process.
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

impl Client {
    /// Start a [`Builder`].
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.inner.api_key
    }

    pub(crate) fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    /// Access the `/users/export/*` endpoints.
    #[cfg(feature = "export")]
    pub fn export(&self) -> crate::export::Export<'_> {
        crate::export::Export::new(self)
    }
}

/// Fluent constructor for [`Client`].
#[derive(Debug, Default)]
pub struct Builder {
    credentials: Option<Credentials>,
}

impl Builder {
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

    /// Build the client. Fails if no credentials were supplied or
    /// `rest_endpoint` cannot be parsed as a URL.
    pub fn build(self) -> Result<Client> {
        let credentials = self.credentials.ok_or(Error::MissingApiKey)?;

        if credentials.api_key.trim().is_empty() {
            return Err(Error::MissingApiKey);
        }

        let mut base_url = Url::parse(&credentials.rest_endpoint)
            .map_err(|source| Error::InvalidEndpoint { source })?;

        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }

        let http = http::build_reqwest_client()?;

        Ok(Client {
            inner: Arc::new(Inner {
                api_key: credentials.api_key,
                base_url,
                http,
            }),
        })
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

    #[cfg(feature = "export")]
    #[test]
    fn builder_normalises_base_path() {
        let client = Client::builder()
            .credentials(Credentials {
                api_key: "k".into(),
                rest_endpoint: "https://rest.iad-01.braze.com".into(),
            })
            .build()
            .expect("client builds");
        assert_eq!(client.base_url().path(), "/");
    }
}
