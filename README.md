![ci](https://github.com/fujiapple852/rust-daml-bindings/actions/workflows/ci.yml/badge.svg)
[![Documentation](https://docs.rs/daml/badge.svg)](https://docs.rs/daml/0.2.2)
[![Crate](https://img.shields.io/crates/v/daml.svg)](https://crates.io/crates/daml/0.2.2)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Rust Bindings for Daml

Unofficial Rust bindings and tools for [Daml](https://daml.com).

## Crates

The project provides the following crates:

| crate                                                       | description                                        |
|-------------------------------------------------------------|----------------------------------------------------|
| [daml](https://crates.io/crates/daml/0.2.2)                 | Daml prelude & common entry point                  |
| [daml-grpc](https://crates.io/crates/daml-grpc/0.2.2)       | Daml Ledger GRPC API bindings                      |
| [daml-json](https://crates.io/crates/daml-json/0.2.2)       | Daml Ledger JSON API bindings                      |
| [daml-codegen](https://crates.io/crates/daml-codegen/0.2.2) | Generate Rust GRPC API bindings from Daml          |
| [daml-derive](https://crates.io/crates/daml-derive/0.2.2)   | Macros for generating Rust GRPC bindings from Daml |
| [daml-macro](https://crates.io/crates/daml-macro/0.2.2)     | Helper macros for working with Daml GRPC values    |
| [daml-util](https://crates.io/crates/daml-util/0.2.2)       | Utilities for working with Daml ledgers            |
| [daml-lf](https://crates.io/crates/daml-lf/0.2.2)           | Library for working with Daml-LF archives          |
| [daml-bridge](https://crates.io/crates/daml-bridge/0.2.2)   | Daml JSON <> GRPC Ledger bridge                    |

## Usage

Applications should always depend on the `daml` crate directly and specify the appropriate features to enable the
required functionality:

```toml
[dependencies]
daml = { version = "0.2.2", features = ["full"] }
```

See the [documentation](https://docs.rs/daml/0.2.2) for the full set of feature flags available.

## Example Applications

Several example applications are available in
the [`examples`](https://github.com/fujiapple852/rust-daml-bindings/tree/master/examples) directory showcasing various
features of the library. Additionally, most crates provide comprehensive integration tests which demonstrate usage.

## Minimum Supported Rust Version

The current MSRV is 1.59.0.

## Supported Daml Version

This library has been tested against Daml-LF version `1.14` and Daml Connect SDK `1.18.1`.

## Changelog

Please see the [CHANGELOG](https://github.com/fujiapple852/rust-daml-bindings/blob/master/CHANGELOG.md) for a release
history.

## License

This library is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022