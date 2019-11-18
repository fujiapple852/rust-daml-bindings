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
The current implementation uses the [grpc-rs](https://github.com/pingcap/grpc-rs) library which is a thin Rust wrapper 
over [gRPC Core](https://github.com/grpc/grpc) and so brings with it several build-time dependencies which are listed 
below.  This library will be migrated to use [tower-grpc](https://github.com/tower-rs/tower-grpc) which is a pure Rust 
implementation built on top of [PROST!](https://github.com/danburkert/prost) which does not have these dependencies.

- Rust (stable) >= 1.38.0
- Protoc >= 3.1.0
- CMake >= 3.8.0
- binutils >= 2.22
- Go >=1.7
- DA SDK (>=0.13.27)

## Crates
The project provides the following crates:

| crate                      | description                                 | status      |
|----------------------------|---------------------------------------------|-------------|
| daml                       | DAML prelude & common entry point           | alpha       |
| daml_ledger_api            | Basic DAML Ledger API binding in Rust       | alpha       |
| daml_ledger_codegen        | Rust codegen for DAML archives              | alpha       |
| daml_ledger_codegen_derive | Custom attribute for Rust DAML codegen      | alpha       |
| daml_ledger_derive         | Custom attributes for Rust<>DAML conversion | alpha       |
| daml_ledger_ffi            | FFI wrapper for C-style integration         | not started |
| daml_ledger_macro          | Macros to create and extract DAML value     | alpha       |
| daml_lf                    | Read Dar and Dalf files & bytes             | alpha       | 
| ping_pong                  | Standalone application using the ledger api | alpha       |
| rental                     | Standalone app using the codegen            | not started |

## Build
Standard Cargo debug/release build steps:

```
$ cd rust-api-bindings
$ cargo build
```

The build will trigger the generation of the `daml_ledger_api/src/grpc_protobuf_autogen` code from the protobuf 
source files in `daml_ledger_client/resources/protobuf`.  Note that if you need to rebuild these Rust source files you
can do so by touching `build.rs` and rerunning the cargo build.

## Features
The API has a `testing` feature flag to control whether the testing-only GRPC services (`TimeService` & `ResetService`) are 
built or not.  The feature is enabled by default but can be disabled for production builds by using the 
`--no-default-features` switch:

```
cargo build --no-default-features
```

Note that the `testing` feature is required for the integration tests so that they can use the `ResetService`.

## Run the Integration Tests
The integration tests run against two instances of the DA Sandbox, one in `Static` time mode (on port `8081`) and one 
in `Wallclock` time mode (on port `8080`).  The tests assume that the standard `PingPong` module is loaded.  For 
convenience that module is bundled with this library and both needed sandboxes can be started up as follows:

```
$ cd rust-api-bindings/resources/testing_types_sandbox
$ make run
```

## Run All Tests
To run all tests (unit, integration & doc tests):

```
$ cd rust-api-bindings
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
$ cd rust-api-bindings
$ cargo build --release
# target/release/ping_pong_sample_app
```

## Clippy
Clippy is used for linting and is set per module to be `#![warn(clippy::all, clippy::pedantic)]` with local overrides 
throughout the source where needed.  Clippy can be run with:

```
$ cd rust-api-bindings
$ cargo clippy --workspace
$ cargo clippy --workspace --tests
```

## Format
The code can be automatically formatted as follows (requires nightly Rust)::

```
$ cd rust-api-bindings
$ cargo +nightly fmt
```

See `rustfmt.toml` for configuration settings.

## Doc
Rust docs can be generated as follows (requires nightly Rust):

```
$ cd rust-api-bindings
$ cargo +nightly doc --all --no-deps --open
```

## Doctests

```
$ cd rust-api-bindings
$ cargo test --doc --workspace
```

The generated docs can be accessed from `target/doc/daml_ledger_api/index.html`

## Library Upgrade
To check for outdated dependencies with [cargo outdated](https://github.com/kbknapp/cargo-outdated):

```
$ cargo outdated -R
``` 