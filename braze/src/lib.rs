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
//! # async fn run() -> Result<(), braze::Error> {
//! use braze::{Client, export::ExportUsersByIdsRequest};
//!
//! let client = Client::builder()
//!     .credentials_file(std::env::var("BRAZE_CREDENTIALS").unwrap())?
//!     .build()?;
//!
//! let response = client
//!     .export()
//!     .users_by_ids(&ExportUsersByIdsRequest {
//!         external_ids: Some(vec!["user_1".into()]),
//!         fields_to_export: Some(vec!["email".into(), "first_name".into()]),
//!         ..Default::default()
//!     })
//!     .send()
//!     .await?;
//!
//! println!("{} users returned", response.users.len());
//! # Ok(()) }
//! ```
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
//! - `export` (default) — `/users/export/*` endpoints.
//! - `trace` — `#[tracing::instrument]` on public async methods.

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]

/// Default Braze REST API surface targeted by this crate.
///
/// Braze does not version its REST API in the URL path, so this constant is
/// informational only — it tracks the version of the upstream documentation
/// the typed wrappers were last reconciled against.
pub const DEFAULT_API_VERSION: &str = "2025-10";

mod client;
mod error;

mod http;

#[cfg(feature = "export")]
pub mod export;

pub use client::{Builder, Client, Credentials};
pub use error::{Error, Result};

/// Re-export of the raw `progenitor`-generated client.
///
/// Use this when an endpoint isn't yet covered by an ergonomic wrapper.
pub use braze_generated as generated;
