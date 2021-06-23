# Build & run

How to build and run `daml2oas` as a standalone (using `musl`) executable (run from `rust-daml-bindings/examples/daml2oas`).

## Build the Docker image

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

# Companion File

TODO

## Format

TODO

## Example

```yaml
summary: MyApp Daml JSON API
description: OpenAPI specification for MyApp Daml JSON API
version: 1.9.9
contact:
  name: Bob Smith
  url: https://example.com/myapp
  email: bob.smith@example.com
servers:
  - http://localhost:7575
```

# Render

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
      apiDescriptionUrl="https://gist.githubusercontent.com/fujiapple852/0bc8784d215bca3f8b399123588c9c77/raw/c173bf7c9a677c899737382e59a37be22791c844/openapi.json"
      router="hash"
      layout="sidebar"
    />

  </body>
</html>
```