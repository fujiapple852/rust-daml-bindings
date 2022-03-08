[![Documentation](https://docs.rs/daml-bridge/badge.svg)](https://docs.rs/daml-bridge/0.2.2)
[![Crate](https://img.shields.io/crates/v/daml-bridge.svg)](https://crates.io/crates/daml-bridge/0.2.2)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Daml Bridge

This crate provides a bridge between the Daml JSON and Daml GRPC APIs.

## Install

```shell
cargo install daml-bridge
```

## Usage

```shell
USAGE:
    daml-bridge [OPTIONS] --ledger-uri <uri> --http-port <port> --bridge-token <token>

OPTIONS:
        --bridge-token <token>
            The JWT token the bridge will use for package refresh from the ledger server

        --encode-decimal-as-string
            Sets whether decimal values are encoded as JSON strings

        --encode-int64-as-string
            Sets whether int64 values are encoded as JSON strings

    -h, --help
            Print help information

        --http-host <host>
            The host the http server should listen on [default: 127.0.0.1]

        --http-port <port>
            The port the http server should listen on

        --ledger-connect-timeout <duration>
            The ledger server connection timeout [default: 5s]

        --ledger-timeout <duration>
            The ledger server timeout [default: 5s]

        --log-filter <log-filter>
            Sets the log filters [default: daml-bridge=info]

        --package-reload-interval <interval>
            How frequently the bridge should refresh the Daml packages from the ledger server
            [default: 5s]

    -s, --ledger-uri <uri>
            The ledger server GRPC uri (i.e. https://127.0.0.1:7575)

    -V, --version
            Print version information
```

## Example

To run the bridge against a Daml ledger listening on `https://127.0.0.1:6865` and expose the JSON API on port `8080`:

```shell
TOKEN="..."
daml-bridge --ledger-uri https://127.0.0.1:6865 --http-port 8080 --bridge-token $TOKEN
```

## Limitations

The bridge supports all operations of the Daml JSON API except:

- [Queries](https://docs.daml.com/json-api/index.html#get-all-active-contracts)
- [Streaming API](https://docs.daml.com/json-api/index.html#streaming-api)
- [Healthcheck Endpoints](https://docs.daml.com/json-api/index.html#healthcheck-endpoints)

It does not provide a database backing store or cache, all operations are related to the underlying GRPC API.

## License

`daml-bridge` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022