//! # braze
//!
//! Unofficial Rust SDK for Braze APIs.
//!
//! ```no_run
//! use braze::Client;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .api_key(std::env::var("BRAZE_API_KEY")?)
//!     .base_url("https://rest.iad-01.braze.com")
//!     .build()?;
//! # let _ = client;
//! # Ok(())
//! # }
//! ```
//!
//! Per-API examples are in the module docs.
//!
//! The REST API key and the instance's REST endpoint (`base_url`) are under
//! *Settings → APIs and Identifiers* in the Braze dashboard. The SDK takes
//! both as values; where they come from is up to the application. The SDK
//! doesn't retry; see [`Error::is_retryable`] and [`Error::retry_after`].
//!
//! ## Cargo features
//!
//! None are enabled by default. [`generated`] is always compiled in full.
//!
//! - `export`, `messaging` — one per API.
//! - `trace` — `#[tracing::instrument]` on public async methods.
//!
//! ```toml
//! braze = { version = "0.3", features = ["export", "messaging"] }
//! ```

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Re-export, so callers use the same `chrono` version as the generated types.
pub use chrono;

/// Re-export, so callers use the same `reqwest` version as
/// [`Builder::default_headers`] and `.header()` / `.headers()`.
pub use reqwest::header;

/// Re-export; [`Credentials::api_key`] is a [`SecretString`](secrecy::SecretString).
pub use secrecy;

mod client;
mod error;
mod headers;
#[cfg(feature = "_api")]
mod http;
#[cfg(feature = "_api")]
mod request;

#[cfg(feature = "export")]
pub mod export;

#[cfg(feature = "messaging")]
pub mod messaging;

pub use client::{Builder, Client, Credentials};
pub use error::{Error, Result};
#[cfg(feature = "_api")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "export", feature = "messaging"))))]
pub use request::RequestBuilder;

/// The raw `progenitor` crate, for endpoints the wrapper doesn't cover yet.
pub use braze_generated as generated;
