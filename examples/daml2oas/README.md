[![Documentation](https://docs.rs/daml2oas/badge.svg)](https://docs.rs/daml2oas)
[![Crate](https://img.shields.io/crates/v/daml2oas.svg)](https://crates.io/crates/daml2oas)
![maintenance-status](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# daml2oas

Generate OpenAPI and AsyncAPI specification documents for the Daml JSON API from a Dar file.

## Install

```shell
cargo install daml2oas
```

## Usage

```shell
USAGE:
    daml2oas [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    a2s     Generate an AsyncAPI document from the given Dar file
    help    Print this message or the help of the given subcommand(s)
    oas     Generate an OpenAPI document from the given Dar file
```

### OpenAPI Usage

```shell
USAGE:
    daml2oas oas [OPTIONS] <dar>

ARGS:
    <dar>    Sets the input dar file to use

OPTIONS:
    -c, --companion-file <companion-file>                the companion yaml file with auxiliary data to inject into the generated OAS document
    -d, --datadict-file <datadict-file>                  the data dictionary to use to augment the generated JSON schema
        --data-title <data-title>                        include the `title` property describing the data item name (i.e. Foo.Bar:Baz) [default: data] [possible values: none, data]
    -f, --format <format>                                the output format [default: json] [possible values: json, yaml]
    -h, --help                                           Print help information
        --include-archive-choice                         include the Archive choice which is available on every template
        --include-general-operations                     include the general (non-template specific) /v1/create, /v1/exercise, /v1/create-and-exercise & /v1/fetch endpoints
        --include-package-id                             include the package id in fully qualified templates
    -m, --module <module-path>                           module path prefix in the form Foo.Bar.Baz
    -o, --output <output>                                the output file path
    -p, --reference-prefix <reference-prefix>            the prefix for absolute $ref schema references [default: #/components/schemas/]
    -r, --reference-mode <reference-mode>                encode references as as $ref schema links or inline [default: ref] [possible values: ref, inline]
    -s, --path-style <path-style>                        encode paths with fragment (i.e. '#') or slash ('/') [default: fragment] [possible values: fragment, slash]
    -t, --template-filter-file <template-filter-file>    the template filter to apply
        --type-description <type-description>            include the `description` property describing the Daml type [default: all] [possible values: none, data, all]
    -v                                                   Sets the level of verbosity
```

### AsyncAPI Usage

```shell
USAGE:
    daml2oas a2s [OPTIONS] <dar>

ARGS:
    <dar>    Sets the input dar file to use

OPTIONS:
    -c, --companion-file <companion-file>                the companion yaml file with auxiliary data to inject into the generated OAS document
    -d, --datadict-file <datadict-file>                  the data dictionary to use to augment the generated JSON schema
        --data-title <data-title>                        include the `title` property describing the data item name (i.e. Foo.Bar:Baz) [default: data] [possible values: none, data]
    -f, --format <format>                                the output format [default: json] [possible values: json, yaml]
    -h, --help                                           Print help information
        --include-package-id                             include the package id in fully qualified templates
    -m, --module <module-path>                           module path prefix in the form Foo.Bar.Baz
    -o, --output <output>                                the output file path
    -p, --reference-prefix <reference-prefix>            the prefix for absolute $ref schema references [default: #/components/schemas/]
    -r, --reference-mode <reference-mode>                encode references as as $ref schema links or inline [default: ref] [possible values: ref, inline]
    -t, --template-filter-file <template-filter-file>    the template filter to apply
        --type-description <type-description>            include the `description` property describing the Daml type [default: all] [possible values: none, data, all]
    -v                                                   Sets the level of verbosity
```

## Examples

### OpenAPI

Generate an OpenAPI specification for `MyModel.dar` in `json` format:

```shell
daml2oas oas MyModel.dar
```

Generate an OpenAPI specification for `MyModel.dar` in `yaml` format with documentation augmented from
`.companion.yaml`, a data dictionary from `.datadict.yaml` and filtered for the templates and choices specified by 
`.template_filter.yaml`:

```shell
daml2oas oas MyModel.dar -f yaml -c .companion.yaml -d .datadict.yaml -f .template_filter.yaml
```

### AsyncAPI

Generate an AsyncAPI specification for `MyModel.dar` in `json` format:

```shell
daml2oas oas MyModel.dar
```

# Companion File

The companion `yaml` file contains additional documentation about templates and choices as well as other metadata that
will be used to augment the generated OpenAPI and AsyncAPI specifications.

All fields are non-mandatory.

## Example

```yaml
title: MyApp 1.0 API Documentation
summary: MyApp Daml JSON API
description: OpenAPI specification for MyApp Daml JSON API
version: 1.9.9
contact:
  name: Bob Smith
  url: https://example.com/myapp
  email: bob.smith@example.com
servers:
  - http://localhost:7575
operations:
  Fuji.PingPong:Pong:
    create: create a Pong!
    createAndExercise:
      RespondPing: create a Pong and then respond with a Ping
    exerciseById:
      RespondPing: respond with a Ping by id
      Archive: archive the contract by id
    exerciseByKey:
      RespondPing: respond with a Ping by key
      Archive: archive the contract by key
    fetchById: fetch a Pong contract by id from the ledger
    fetchByKey: fetch a Pong contract by key from the ledger
```

# Datadict File

The datadict `yaml` file contains additional documentation for Daml records that will be used to augment the generated
OpenAPI and AsyncAPI specifications.

All fields are non-mandatory.

## Example

```yaml
Fuji.Vehicle:Car:
  title: My Custom Title
  description: Represents a Car
  items:
    driver: the driver of the car
    make: the make of the car
    owner: the owner of the car
    purchase_time: when the car was purchased
    reg_year: the registration year
Fuji.PingPong:Ping:
  description: The demo Ping template
  items:
    count: the number of times ping has ponged
```

# Template Filter File

The template filter `yaml` file allows you to specify a subset of templates and choices to generate.

All fields are non-mandatory.

## Example

```yaml
Fuji.Nested:NestedTemplate: all
Fuji.PingPong:Ping:
  selected:
    - RespondPong
    - ResetPingCount
Fuji.PingPong:Pong:
  selected:
    - RespondPing
Fuji.Shape:CircleTemplate: all
Fuji.DupUsage:DupUsage: all
```

# Render

The generated OpenAPI and AsyncAPI specifications can be rendered with any tool that supports the relevant standard. The
following `html` examples uses the [stoplight.io](https://stoplight.io/) library to render an OpenAPI specification: 

```html
<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
    <title>Elements in HTML</title>
    <!-- Embed elements Elements via Web Component -->
    <script src="https://unpkg.com/@stoplight/elements@beta/web-components.min.js"></script>
    <link rel="stylesheet" href="https://unpkg.com/@stoplight/elements@beta/styles.min.css">
</head>
<body>

<elements-api
        apiDescriptionUrl="https://gist.githubusercontent.com/fujiapple852/openapi-example.json"
        router="hash"
        layout="sidebar"
/>

</body>
</html>
```

# Build Standalone Executable

How to build and run `daml2oas` as a standalone (using `musl`) executable (run
from `rust-daml-bindings/examples/daml2oas`).

## Build the `rust-musl` Docker image

To build the `rust-musl` Docker image:

```shell
make build-image
```

## Build the artifact

To generate the `daml2oas` executable using `musl`:

```shell
make build
```

## Run the artifact

To run the generated artifact on a vanilla `centos` Docker image:

```shell
make run-oas dar=rust/resources/testing_types_sandbox/TestingTypes-latest.dar
```

## License

`daml2oas` is distributed under the terms of the Apache License (Version 2.0).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in time by you, as defined
in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

See [LICENSE](LICENSE) for details.

Copyright 2022