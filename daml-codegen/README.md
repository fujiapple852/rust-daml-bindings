[![Documentation](https://docs.rs/daml-codegen/badge.svg)](https://docs.rs/daml-codegen/0.2.0)
[![Crate](https://img.shields.io/crates/v/daml-codegen.svg)](https://crates.io/crates/daml-codegen/0.2.0)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml Codegen

A library and a tool for generating Rust types from `daml` code.

This crate provides:

- A code generator backend for the of custom attributes and procedural macros defined in
  the [`daml-derive`](https://docs.rs/daml-derive/0.2.0/daml_derive/) crate
- A [`daml_codegen`](https://docs.rs/daml-codegen/0.2.0/daml_codegen/generator/fn.daml_codegen.html) function which is
  designed to be used from `build.rs` files
- A standalone codegen cli tool

## Library

This crate should not be used directly when used as a library, instead you should depend on
the [`daml`](https://crates.io/crates/daml/0.2.0) crate and enable the `codegen` feature:

```toml
[dependencies]
daml = { version = "0.2.0", features = [ "codegen" ] }
```

## Install

```shell
cargo install daml-codegen
```

## Usage

```shell
USAGE:
    daml-codegen [OPTIONS] <dar>

ARGS:
    <dar>    Sets the input Dar file to use

OPTIONS:
    -c, --combine-modules              Combine modules as a single file
    -f, --module-filter <filter>...    Sets the regex module filter to apply
    -h, --help                         Print help information
    -i, --render-intermediate          Generate intermediate types
    -o, --output-dir <output>          Sets the output path
    -v, --verbose                      Sets the level of verbosity
    -V, --version                      Print version information
```

## Example

To generate Rust types from Daml dar `MyModel.dar` in single src file `/tmp/my_model_0_1_0.rs`:

```shell
daml-codegen MyModel.dar --combine-modules -o /tmp/MyModel.dar
```

## License

`daml-codegen` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022