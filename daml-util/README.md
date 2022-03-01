[![Documentation](https://docs.rs/daml-util/badge.svg)](https://docs.rs/daml-util)
[![Crate](https://img.shields.io/crates/v/daml-util.svg)](https://crates.io/crates/daml-util)

# Daml Util

This crate provides helper utilities for working with the Daml Ledgers.

This crate should not be used directly, instead you depend on the [`daml`](https://crates.io/crates/daml) crate and 
enable the `utils` feature:

```yaml
daml = { version = "0.1.0", features = [ "utils" ] }
```

## License

`daml-util` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.

Copyright 2022