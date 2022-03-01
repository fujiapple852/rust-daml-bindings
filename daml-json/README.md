[![Documentation](https://docs.rs/daml-json/badge.svg)](https://docs.rs/daml-json)
[![Crate](https://img.shields.io/crates/v/daml-json.svg)](https://crates.io/crates/daml-json)

# Daml JSON

This crate provides a library for using the Daml JSON API.

This crate should not be used directly, instead you depend on the [`daml`](https://crates.io/crates/daml) crate and 
enable the `json` feature:

```yaml
daml = { version = "0.1.0", features = [ "json" ] }
```

## License

`daml-json` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.

Copyright 2022