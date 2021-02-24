[![CircleCI](https://circleci.com/bb/fujiapple852/rust-daml-bindings.svg?style=svg&circle-token=5c7eace581559ba93ec1f87b563c541622572ab4)](https://circleci.com/bb/fujiapple852/rust-daml-bindings)

# DAML API Rust Bindings 
A Rust implementation for the Digital Asset DAML GRPC ledger [API](https://docs.daml.com/app-dev/ledger-api-introduction/index.html).

## Status
The project is in an early development stage and the API is unstable.  It has not yet been published to [crates.io](https://crates.io/).  
The status of the feature set:

- [x] Support for DAML Ledger GRPC API
- [ ] Support for DAML Ledger JSON API
- [X] Support for http & https (TLS)
- [X] Support for passing JWT bearer tokens (HS256/RS256/EC256 DAML Sandbox token builder provided)
- [X] Fully async API (via `async`/`await`, `std::futures` & `futures` [0.3.x](https://docs.rs/futures/0.3.1/futures/))
- [X] Full suite of Sandbox integration tests
- [X] Macros to create and extract DAML values
- [X] Support for parsing DAML LF (versions `1.6`, `1.7` & `1.8`)
- [X] Code Generator (generate Rust types from DAML LF) 
- [X] Custom attributes for automatic Rust<>DAML conversions
- [X] Sample applications
- [ ] Executor API

## Dependencies
These ledger bindings use the [tonic](https://github.com/hyperium/tonic) GRPC library which in turn uses the 
[PROST!](https://github.com/danburkert/prost) library for generating Rust representations of the DAML ledger API 
protocols buffers.

# Minimum Supported Rust Version
This crate is guaranteed to compile on stable Rust 1.50 and up.

# Supported DAML Version
These bindings support `DAML-LF` versions `1.6`, `1.7`, `1.8` & `1.11` and has been tested against DAML SDKs up to `1.11.1`.

## Crates
The project provides the following crates:

| crate        | description                                 | status      |
|--------------|---------------------------------------------|-------------|
| daml         | DAML prelude & common entry point           | alpha       |
| daml_grpc    | DAML Ledger GRPC API bindings               | beta        |
| daml_json    | Daml Ledger JSON API bindings               | alpha       |
| daml_codegen | Rust codegen for DAML archives              | beta        |
| daml_derive  | Custom attributes for Rust<>DAML conversion | beta        |
| daml_macro   | Macros to create and extract DAML value     | beta        |
| daml_util    | Utilities to aid working with DAML ledgers  | alpha       |
| daml_lf      | Read Dar and Dalf files & bytes             | beta        | 

## Build
Standard Cargo debug/release build steps:

```
$ cd rust-daml-bindings
$ cargo build
```

The build will trigger the generation of the GRPC protobuf code which is included by `daml_grpc/src/grpc_protobuf.rs`.  The protobuf source files are read from `daml_grpc/resources/protobuf`.  Note that if you need to rebuild these 
Rust source files you can do so by touching `build.rs` and rerunning the cargo build.

## Features
The API has a `sandbox` feature flag to control whether the testing-only GRPC services (`TimeService` & `ResetService`) are 
built or not.  The feature is disabled by default and must be enabled for integration tests.

```
daml_grpc = { version = "0.1", features = [ "sandbox" ] }
```

The `admin` feature flag can be enabled to include the package and party management services.

```
daml_grpc = { version = "0.1", features = [ "admin" ] }
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
$ target/release/ping_pong_demo
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

The generated docs can be accessed from `target/doc/daml_grpc/index.html`

## Library Upgrade
To check for outdated dependencies with [cargo outdated](https://github.com/kbknapp/cargo-outdated):

```
$ cargo outdated -R
``` 