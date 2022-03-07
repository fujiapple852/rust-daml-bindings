[![Documentation](https://docs.rs/daml-util/badge.svg)](https://docs.rs/daml-util/0.2.0)
[![Crate](https://img.shields.io/crates/v/daml-util.svg)](https://crates.io/crates/daml-util/0.2.0)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml Util

This crate provides helper utilities for working with the Daml Ledgers.

This crate should not be used directly, instead you should depend on the [`daml`](https://crates.io/crates/daml/0.2.0)
crate and enable the `utils` feature:

```toml
[dependencies]
daml = { version = "0.2.0", features = [ "utils" ] }
```

## License

`daml-util` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022