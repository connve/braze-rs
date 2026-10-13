//! # braze
//!
//! Unofficial Rust SDK for Braze APIs.
//!
//! The crate wraps a `progenitor`-generated client (re-exported as
//! [`generated`]) and adds ergonomic auth, error handling, and per-endpoint
//! modules.
//!
//! ## Quick start
//!
//! ```no_run
//! use braze::Client;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder()
//!     .credentials_file(std::env::var("BRAZE_CREDENTIALS")?)?
//!     .build()?;
//! # let _ = client;
//! # Ok(())
//! # }
//! ```
//!
//! See module-level docs for per-API examples: `export`, `messaging`.
//!
//! ## Authentication
//!
//! Braze uses a static REST API key passed as `Authorization: Bearer <key>`.
//! The key and the per-instance `rest_endpoint` come from the Braze dashboard
//! under *Settings → API Keys*. Both can be loaded from a JSON file pointed to
//! by the `BRAZE_CREDENTIALS` environment variable — see
//! [`Credentials::from_file`].
//!
//! ## Cargo features
//!
//! Each API surface is gated behind a feature. None are enabled by default.
//! The generated client in [`generated`] is always compiled in full.
//!
//! - `export` — `/users/export/*` endpoints.
//! - `messaging` — messaging endpoints (`/messages/send`, `/campaigns/trigger/send`, etc.).
//! - `trace` — `#[tracing::instrument]` on public async methods; pulls in `tracing`.
//!
//! ```toml
//! braze = { version = "0.2", features = ["export", "messaging"] }
//! ```

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Re-export of [`chrono`], which appears in this crate's public API.
///
/// Depend on this re-export rather than adding `chrono` separately, so the
/// version stays in step with the one the SDK was built against.
pub use chrono;

/// Re-export of [`reqwest::header`], whose types appear in this crate's public
/// API ([`Builder::default_headers`] and the `.header()` / `.headers()`
/// methods on every request builder).
///
/// Depend on this re-export rather than adding `reqwest` separately, so the
/// version stays in step with the one the SDK was built against.
pub use reqwest::header;

/// Re-export of [`secrecy`], whose [`SecretString`](secrecy::SecretString)
/// holds [`Credentials::api_key`].
pub use secrecy;

/// Default Braze REST API surface targeted by this crate.
///
/// Braze does not version its REST API in the URL path, so this constant is
/// informational only — it tracks the version of the upstream documentation
/// the typed wrappers were last reconciled against.
pub const DEFAULT_API_VERSION: &str = "2025-10";

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

/// Re-export of the raw `progenitor`-generated client.
///
/// Use this when an endpoint isn't yet covered by an ergonomic wrapper.
pub use braze_generated as generated;
