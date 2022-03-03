![ci](https://github.com/fujiapple852/rust-daml-bindings/actions/workflows/ci.yml/badge.svg)
[![Documentation](https://docs.rs/daml/badge.svg)](https://docs.rs/daml)
[![Crate](https://img.shields.io/crates/v/daml.svg)](https://crates.io/crates/daml)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Rust Bindings for Daml

Unofficial Rust bindings and tools for [Daml](https://daml.com).

## Crates

The project provides the following crates:

| crate                                                 | description                                        |
|-------------------------------------------------------|----------------------------------------------------|
| [daml](https://crates.io/crates/daml)                 | Daml prelude & common entry point                  |
| [daml-grpc](https://crates.io/crates/daml-grpc)       | Daml Ledger GRPC API bindings                      |
| [daml-json](https://crates.io/crates/daml-json)       | Daml Ledger JSON API bindings                      |
| [daml-codegen](https://crates.io/crates/daml-codegen) | Generate Rust GRPC API bindings from Daml          |
| [daml-derive](https://crates.io/crates/daml-derive)   | Macros for generating Rust GRPC bindings from Daml |
| [daml-macro](https://crates.io/crates/daml-macro)     | Helper macros for working with Daml GRPC values    |
| [daml-util](https://crates.io/crates/daml-util)       | Utilities for working with Daml ledgers            |
| [daml-lf](https://crates.io/crates/daml-lf)           | Library for working with Daml-LF archives          |
| [daml-bridge](https://crates.io/crates/daml-bridge)   | Daml JSON <> GRPC Ledger bridge                    |

## Usage

Applications should always depend on the `daml` crate directly and specify the appropriate features to enable the
required functionality:

```toml
daml = { version = "0.1.1", features = [ "full" ] }
```

See the [documentation](https://docs.rs/daml) for the full set of feature flags available.

## Example Applications

Several example applications are available in the [`examples`](examples/) directory showcasing various features of the
library.  Additionally, most crates provide comprehensive integration tests which demonstrate usage. 

## Minimum Supported Rust Version

The current MSRV is 1.59.0.

## Supported Daml Version

This library has been tested against Daml-LF version `1.14` and Daml Connect SDK `1.18.1`.

## License

This library is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022