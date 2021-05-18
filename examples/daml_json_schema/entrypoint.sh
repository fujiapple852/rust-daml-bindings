#!/bin/bash -x
cd /rust/examples/daml_json_schema
cargo build --release --target x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/daml_json_schema
