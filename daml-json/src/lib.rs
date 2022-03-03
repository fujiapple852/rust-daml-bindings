//! Daml ledger JSON [API](https://docs.daml.com/json-api/index.html).
//!
//! A simple Daml ledger JSON API client.
//!
//! # Examples
//!
//! The following example connects to a Daml ledger and creates a contract.
//!
//! ```no_run
//! use serde_json::json;
//! use daml_json::service::DamlJsonClientBuilder;
//! use daml_json::error::DamlJsonResult;
//! #[tokio::main]
//! async fn main() -> DamlJsonResult<()> {
//!     let payload = json!({ "sender": "Alice", "receiver": "Bob", "count": "0" });
//!     let client = DamlJsonClientBuilder::url("https://api.myledger.org").build()?;
//!     let create_response = client.create("Fuji.PingPong:Ping", payload.clone()).await?;
//!     assert_eq!(create_response.payload, payload);
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::future_not_send,
    clippy::missing_const_for_fn,
    clippy::match_wildcard_for_single_variants,
    clippy::similar_names,
    clippy::return_self_not_must_use
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml-json/0.1.1")]

/// Daml JSON API service endpoints.
pub mod service;

/// Daml JSON API request & response types.
pub mod request;

/// Daml JSON API data types.
pub mod data;

/// Daml JSON API errors.
pub mod error;

/// Daml JSON value decoder.
pub mod value_decode;

/// Daml JSON value encoder.
pub mod value_encode;

/// Daml JSON request converter.
pub mod request_converter;

/// Daml JSON response converter.
pub mod response_converter;

/// Daml JSON API data types.
mod schema_data;

/// Daml JSON schema encoder.
pub mod schema_encoder;

mod util;

#[cfg(test)]
mod test_util;
