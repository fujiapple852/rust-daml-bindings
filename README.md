[![CircleCI](https://circleci.com/bb/fujiapple852/rust-daml-bindings.svg?style=svg&circle-token=5c7eace581559ba93ec1f87b563c541622572ab4)](https://circleci.com/bb/fujiapple852/rust-daml-bindings)

# DAML API Rust Bindings 
A Rust implementation for the Digital Asset DAML GRPC ledger [API](https://docs.daml.com/app-dev/ledger-api-introduction/index.html).

## Status
The project is in an early development stage and the API is unstable.  It has not yet been published to [crates.io](https://crates.io/).  
The status of the feature set:

- [x] Support for all DAML Ledger API (v1) GRPC services
- [X] Support for async I/O and streams (via `futures` [0.1.x](https://docs.rs/futures/0.1.25/futures/))
- [X] Sample Application and full suite of Sandbox integration tests
- [X] Macros to create and extract DAML values
- [X] Support for DAML LF
- [X] Custom Attributes (for automatic Rust<>DAML conversions)
- [X] Code Generator (from DAML LF)
- [ ] Executor API
- [ ] Client Authentication
- [ ] FFI Wrapper (C interface)

## Prerequisites
These ledger bindings use the [tonic](https://github.com/hyperium/tonic) GRPC library which in turn uses the 
[PROST!](https://github.com/danburkert/prost) library for generating Rust representations of the DAML ledger API 
protocols buffers.

# Minimum Supported Rust Version
This crate is guaranteed to compile on stable Rust 1.40 and up.

# Supported DAML Version
These bindings support `DAML-LF` versions `1.6` & `1.7` and has been tested against DAML SDKs up to `0.13.46`.

## Crates
The project provides the following crates:

| crate               | description                                 | status      |
|---------------------|---------------------------------------------|-------------|
| daml                | DAML prelude & common entry point           | alpha       |
| daml_ledger_api     | Basic DAML Ledger API binding in Rust       | alpha       |
| daml_ledger_codegen | Rust codegen for DAML archives              | alpha       |
| daml_ledger_derive  | Custom attributes for Rust<>DAML conversion | alpha       |
| daml_ledger_ffi     | FFI wrapper for C-style integration         | not started |
| daml_ledger_macro   | Macros to create and extract DAML value     | alpha       |
| daml_ledger_util    | Utilities to aid working with DAML ledgers  | alpha       |
| daml_lf             | Read Dar and Dalf files & bytes             | alpha       | 
| ping_pong_demo      | Standalone application using the ledger api | alpha       |
| rental_demo         | Standalone app using the codegen            | alpha       |

## Build
Standard Cargo debug/release build steps:

```
$ cd rust-daml-bindings
$ cargo build
```

The build will trigger the generation of the GRPC protobuf code which is included by `daml_ledger_api/src/grpc_protobuf.rs`.  The protobuf source files are read from `daml_ledger_client/resources/protobuf`.  Note that if you need to rebuild these 
Rust source files you can do so by touching `build.rs` and rerunning the cargo build.

## Features
The API has a `testing` feature flag to control whether the testing-only GRPC services (`TimeService` & `ResetService`) are 
built or not.  The feature is disabled by default and must be enabled for integration tests.

```
daml_ledger_api = { version = "0.1", features = [ "testing" ] }
```

The `admin` feature flag can be enabled to include the package and party management services.

```
daml_ledger_api = { version = "0.1", features = [ "admin" ] }
```

## Run the Integration Tests
The integration tests run against two instances of the DA Sandbox, one in `Static` time mode (on port `8081`) and one 
in `Wallclock` time mode (on port `8080`).  The tests assume that the standard `PingPong` module is loaded.  For 
convenience that module is bundled with this library and both needed sandboxes can be started up as follows:

```
$ cd rust-daml-bindings/resources/testing_types_sandbox
$ make run
```

## Run All Tests
To run all tests (unit, integration & doc tests):

```
$ cd rust-daml-bindings
$ cargo test --workspace
```

## Run the Sample Application
A sample standalone `PingPong` application is available in `ping_pong_sample_app`.  To build and run the sample 
application via cargo:

```
# build and run via cargo:
$ cargo run --package ping_pong_sample_app --release
```

To run directly outside of cargo:

```
$ cd rust-daml-bindings
$ cargo build --release
# target/release/ping_pong_demo
```

## Clippy
Clippy is used for linting and is set per module to be `#![warn(clippy::all, clippy::pedantic)]` with local overrides 
throughout the source where needed.  Clippy can be run with:

```
$ cd rust-daml-bindings
$ cargo clippy --workspace
$ cargo clippy --workspace --tests
```

## Format
The code can be automatically formatted as follows (requires nightly Rust)::

```
$ cd rust-daml-bindings
$ cargo +nightly fmt --all
```

See `rustfmt.toml` for configuration settings.

## Doc
Rust docs can be generated as follows (requires nightly Rust):

```
$ cd rust-daml-bindings
$ cargo +nightly doc --all --no-deps --open
```

## Doctests

```
$ cd rust-daml-bindings
$ cargo test --doc --workspace
```

The generated docs can be accessed from `target/doc/daml_ledger_api/index.html`

## Library Upgrade
To check for outdated dependencies with [cargo outdated](https://github.com/kbknapp/cargo-outdated):

```
$ cargo outdated -R
``` 