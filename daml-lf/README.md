[![Documentation](https://docs.rs/daml-lf/badge.svg)](https://docs.rs/daml-lf)
[![Crate](https://img.shields.io/crates/v/daml-lf.svg)](https://crates.io/crates/daml-lf)

# Daml Codegen

This crate provides a library for working with `Daml-LF` packages.

This crate should not be used directly, instead you depend on the [`daml`](https://crates.io/crates/daml) crate and 
enable the `lf` or `lf-full` features:

```yaml
daml = { version = "0.1.0", features = [ "lf" ] }
```

## License

`daml-lf` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.

Copyright 2022