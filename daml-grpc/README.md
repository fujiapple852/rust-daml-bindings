[![Documentation](https://docs.rs/daml-grpc/badge.svg)](https://docs.rs/daml-grpc)
[![Crate](https://img.shields.io/crates/v/daml-grpc.svg)](https://crates.io/crates/daml-grpc)

# Daml GRPC

This crate provides a library for using the Daml GRPC API.

This crate should not be used directly, instead you depend on the [`daml`](https://crates.io/crates/daml) crate and 
enable the `grpc` feature:

```yaml
daml = { version = "0.1.0", features = [ "grpc" ] }
```

## License

`daml-grpc` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.

Copyright 2022