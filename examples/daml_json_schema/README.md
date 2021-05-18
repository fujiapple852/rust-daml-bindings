# Build & run

How to build and run `daml_json_schema` as a standalone (using `musl`) executable.

## Build the Docker image

To build the `rust-musl` Docker image (run from `rust-daml-bindings/examples/daml_json_schema`):

```shell
make build
```

## Build the artifact

To generate the `daml_json_schema` executable using `musl` (run from `rust-daml-bindings/examples/daml_json_schema`):

```shell
docker run -it -v (pwd)../../../:/rust --name daml-json-schema-build --rm fujiapple/rust-musl:latest
```

## Run the artifact

To run the generated artifact on a vanilla `centos` Docker image:

```shell
docker run -it --rm -v(pwd):/rust centos /rust/target/x86_64-unknown-linux-musl/release/daml_json_schema
```