//! Daml ledger JSON [API](https://docs.daml.com/json-api/index.html).
//!
//! A library for working with Daml JSON.
//!
//! This includes:
//! - A Daml JSON API [client](service::DamlJsonClient) and [builder](service::DamlJsonClientBuilder)
//! - A [`DamlValue`](`daml_grpc::data::value::DamlValue`) <> JSON [`Value`](`serde_json::Value`)
//!   [encoder](value_encode::JsonValueEncoder) and [decoder](value_decode::JsonValueDecoder)
//! - A Daml JSON API [`request`](request) to GRPC API [`command`](daml_grpc::data::command)
//!   [converter](request_converter::JsonToGrpcRequestConverter)
//! - A Daml GRPC API [`event`](daml_grpc::data::event) to JSON API [`response`](request)
//!   [converter](response_converter::GrpcToJsonResponseConverter)
//! - A JSON Schema [encoder](schema_encoder::JsonSchemaEncoder)

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
#![doc(html_root_url = "https://docs.rs/daml-json/0.2.2")]

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
