[![Documentation](https://docs.rs/daml-derive/badge.svg)](https://docs.rs/daml-derive)
[![Crate](https://img.shields.io/crates/v/daml-derive.svg)](https://crates.io/crates/daml-derive)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml Derive

This crate provides procedural macros for generating Rust types from `daml`.

This crate should not be used directly, instead you should depend on the [`daml`](https://crates.io/crates/daml) crate
and enable the `derive` feature:

```yaml
daml = { version = "0.1.1", features = [ "derive" ] }
```

## License

`daml-derive` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022