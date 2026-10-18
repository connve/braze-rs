//! The request builder returned by every endpoint method.

use std::fmt;
use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::client::Client;
use crate::error::Result;
use crate::header::{HeaderMap, HeaderName, HeaderValue};
use crate::headers::HeaderBag;

/// A request to one Braze endpoint, dispatched with [`send`](Self::send).
/// Endpoint methods return it under a per-endpoint alias.
#[must_use = "request builders do nothing until `.send().await` is called"]
#[cfg_attr(docsrs, doc(cfg(any(feature = "export", feature = "messaging"))))]
pub struct RequestBuilder<'a, 'r, Req, Resp> {
    client: &'a Client,
    segments: Vec<&'r str>,
    body: &'r Req,
    headers: HeaderBag,
    response: PhantomData<fn() -> Resp>,
}

// Not derived, so rustdoc shows the public features instead of `_api`.
#[cfg_attr(docsrs, doc(cfg(any(feature = "export", feature = "messaging"))))]
impl<Req: fmt::Debug, Resp> fmt::Debug for RequestBuilder<'_, '_, Req, Resp> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestBuilder")
            .field("client", self.client)
            .field("segments", &self.segments)
            .field("body", self.body)
            .field("headers", &self.headers)
            .finish()
    }
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "export", feature = "messaging"))))]
impl<'a, 'r, Req, Resp> RequestBuilder<'a, 'r, Req, Resp> {
    pub(crate) fn new(client: &'a Client, segments: Vec<&'r str>, body: &'r Req) -> Self {
        Self {
            client,
            segments,
            body,
            headers: HeaderBag::default(),
            response: PhantomData,
        }
    }

    /// Adds a header, overriding a client default of the same name. An
    /// invalid or reserved header (see
    /// [`Builder::default_headers`](crate::Builder::default_headers)) fails
    /// [`send`](Self::send) with
    /// [`Error::InvalidHeader`](crate::Error::InvalidHeader).
    pub fn header<N, V>(mut self, name: N, value: V) -> Self
    where
        N: TryInto<HeaderName> + AsRef<str>,
        V: TryInto<HeaderValue>,
    {
        self.headers.add(name, value);
        self
    }

    /// Merges `headers` like `reqwest::RequestBuilder::headers`: names in
    /// `headers` replace earlier values, other names are kept.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.merge(headers);
        self
    }
}

#[cfg_attr(docsrs, doc(cfg(any(feature = "export", feature = "messaging"))))]
impl<Req, Resp> RequestBuilder<'_, '_, Req, Resp>
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    /// Sends the request.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn send(self) -> Result<Resp> {
        crate::http::post_json(
            self.client,
            &self.segments,
            self.headers.finish()?,
            self.body,
        )
        .await
    }
}
