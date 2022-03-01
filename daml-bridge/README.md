[![Documentation](https://docs.rs/daml-bridge/badge.svg)](https://docs.rs/daml-bridge)
[![Crate](https://img.shields.io/crates/v/daml-bridge.svg)](https://crates.io/crates/daml-bridge)

# Daml Bridge

This crate provides a bridge between the Daml JSON and Daml GRPC APIs.

## Example usage

To run the bridge:

```shell
TOKEN="..."
daml-bridge --ledger-uri https://127.0.0.1:8080 --http-port 3030 --bridge-token $TOKEN --log-filter "daml_bridge=trace"
```

## License

`daml-bridge` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.

Copyright 2022