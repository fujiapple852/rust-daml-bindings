[![Documentation](https://docs.rs/daml-grpc/badge.svg)](https://docs.rs/daml-grpc)
[![Crate](https://img.shields.io/crates/v/daml-grpc.svg)](https://crates.io/crates/daml-grpc)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml GRPC

This crate provides a library for using the Daml GRPC API.

This crate should not be used directly, instead you should depend on the [`daml`](https://crates.io/crates/daml) crate
and enable the `grpc` feature:

```yaml
daml = { version = "0.1.1", features = [ "grpc" ] }
```

## License

`daml-grpc` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022