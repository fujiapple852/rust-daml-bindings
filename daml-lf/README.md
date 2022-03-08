[![Documentation](https://docs.rs/daml-lf/badge.svg)](https://docs.rs/daml-lf/0.2.2)
[![Crate](https://img.shields.io/crates/v/daml-lf.svg)](https://crates.io/crates/daml-lf/0.2.2)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml LF

This crate provides a library for working with `Daml-LF` packages.

This crate should not be used directly, instead you should depend on the [`daml`](https://crates.io/crates/daml/0.2.2)
crate and enable the `lf` or `lf-full` features:

```toml
[dependencies]
daml = { version = "0.2.2", features = [ "lf" ] }
```

## License

`daml-lf` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022