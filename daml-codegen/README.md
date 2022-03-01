[![Documentation](https://docs.rs/daml-codegen/badge.svg)](https://docs.rs/daml-codegen)
[![Crate](https://img.shields.io/crates/v/daml-codegen.svg)](https://crates.io/crates/daml-codegen)

# Daml Codegen

This crate provides a library for generating Rust types from `daml` code.

This crate should not be used directly, instead you depend on the [`daml`](https://crates.io/crates/daml) crate and 
enable the `codegen` feature:

```yaml
daml = { version = "0.1.0", features = [ "codegen" ] }
```

## License

`daml-codegen` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.

Copyright 2022