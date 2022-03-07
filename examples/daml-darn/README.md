[![Documentation](https://docs.rs/daml-darn/badge.svg)](https://docs.rs/daml-darn/0.2.1)
[![Crate](https://img.shields.io/crates/v/daml-darn.svg)](https://crates.io/crates/daml-darn/0.2.1)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# Darn

Tools for working with Daml Archives and ledgers.

## Install

```shell
cargo install daml-darn
```

## Usage

```shell
USAGE:
    daml-darn [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help       Print this message or the help of the given subcommand(s)
    intern     Show interned strings and dotted names in a dar
    package    Show dar package details
    token      Generate a Daml sandbox token
```

### Package Usage 

```shell
Show dar package details

USAGE:
    daml-darn package <dar>

ARGS:
    <dar>    Sets the input dar file to use

OPTIONS:
    -h, --help    Print help information
```

### Token Usage

```shell
USAGE:
    daml-darn token [OPTIONS] --token-type <token-type> --key-file <filename> --ledger-id <ledger-id> <--expiry <timestamp>|--duration <seconds>>

OPTIONS:
    -e, --expiry <timestamp>
            Sets the token expiry time (unix timestamp)

    -d, --duration <seconds>
            Sets the duration of the token (seconds)

    -t, --token-type <token-type>
            Sets the token type [possible values: rs256, es256]

    -k, --key-file <filename>
            The file to use to sign the token

    -l, --ledger-id <ledger-id>
            Sets the token ledgerId

    -P, --participant-id <participant-id>
            Sets the token participantId

    -A, --application-id <application-id>
            Sets the token applicationId

    -a, --act-as <party>...
            Sets the token actAs list

    -r, --read-as <party>...
            Sets the token readAs list

    -S, --admin
            Sets the token admin flag

    -o, --output <output>
            Sets the output format [default: token] [possible values: token, json, both]

    -h, --help
            Print help information
```

### Intern Usage

```shell
USAGE:
    daml-darn intern [OPTIONS] <--string|--dotted> <dar>

ARGS:
    <dar>    Sets the input dar file to use

OPTIONS:
    -d, --dotted            Show interned dotted names
    -f, --show-mangled      show mangled names
    -h, --help              Print help information
    -i, --index <index>     the intern indices
        --order-by-index    order by index
        --order-by-name     order by name
    -s, --string            Show interned strings
```

## Examples

### List packages

```shell
daml-darn package MyModel.dar
```

Outputs (abridged):

```
+--------------+---------+--------------------------------------+-------+
| name         | version | package_id                           | lf    |
+--------------+---------+--------------------------------------+-------+
| daml-script  | 1.18.1  | 0323a524706f5ab0c24f300e468798535... | v1.14 |
| MyModel      | 1.9.0   | 80e685325dd4ffe4ed4bd5485fbb33134... | v1.14 |
| daml-stdlib  | 1.18.1  | 9de3ae0b74d8a8fb9589b5f8c893fea5b... | v1.14 |
+--------------+---------+--------------------------------------+-------+
```

### Generate Token

```shell
daml-darn token --key-file es256.key --ledger-id my-ledger --token-type es256 --duration 5000000 --admin
```

Outputs (abridged):

```
eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiJ9.eyJod...
```

### Show interned data

```shell
daml-darn intern -d MyModel.dar
```

Outputs (abridged):

```
+-------+----------------------+------------------------------+
| index | rendered             | segments                     |
+-------+----------------------+------------------------------+
| 610   | Fuji.PingPong        | Fuji(0).PingPong(942)        |
| 497   | Fuji.RentDemo        | Fuji(0).RentDemo(796)        |
| 369   | Fuji.Shape           | Fuji(0).Shape(612)           |
| 229   | Fuji.VariantExamples | Fuji(0).VariantExamples(409) |
| 833   | Fuji.Vehicle         | Fuji(0).Vehicle(1220)        |
+-------+----------------------+------------------------------+
```

## License

`daml-darn` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022