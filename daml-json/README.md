[![Documentation](https://docs.rs/daml-json/badge.svg)](https://docs.rs/daml-json)
[![Crate](https://img.shields.io/crates/v/daml-json.svg)](https://crates.io/crates/daml-json)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml JSON

This crate provides a library for using the Daml JSON API.

This crate should not be used directly, instead you should depend on the [`daml`](https://crates.io/crates/daml) crate
and enable the `json` feature:

```yaml
daml = { version = "0.1.1", features = [ "json" ] }
```

## License

`daml-json` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022