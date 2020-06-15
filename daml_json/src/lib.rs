//! DAML ledger JSON [API](https://docs.daml.com/json-api/index.html).
//!
//! # Examples
//!
//! ```no_run
//! use serde_json::json;
//! use daml_json::service::DamlJsonClientBuilder;
//! use daml_json::error::DamlJsonResult;
//! #[tokio::main]
//! async fn main() -> DamlJsonResult<()> {
//!     let client = DamlJsonClientBuilder::url("https://api.myledger.org").build()?;
//!     let create_response = client.create("DA.PingPong:Ping", json!({ "sender": "Alice", "receiver": "Bob", "count": 0 })).await?;
//!     assert_eq!(create_response.payload, json!({ "sender": "Alice", "receiver": "Bob", "count": "0" }));
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::filter_map,
    clippy::future_not_send,
    clippy::missing_const_for_fn
)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]

/// DAML JSON API service endpoints.
pub mod service;

/// DAML JSON API request & response types.
pub mod request;

/// DAML JSON API data types.
pub mod data;

/// DAML JSON API errors.
pub mod error;

/// DAML JSON decoder.
pub mod decode;

/// DAML JSON encoder.
pub mod encode;

mod util;
