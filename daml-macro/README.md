[![Documentation](https://docs.rs/daml-macro/badge.svg)](https://docs.rs/daml-macro)
[![Crate](https://img.shields.io/crates/v/daml-macro.svg)](https://crates.io/crates/daml-macro)

# Daml Macro

This crate provides helper macros for working with the Daml Ledger GRPC API.

This crate should not be used directly, instead you depend on the [`daml`](https://crates.io/crates/daml) crate and 
enable the `macros` feature:

```yaml
daml = { version = "0.1.0", features = [ "macros" ] }
```

## License

`daml-macro` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022