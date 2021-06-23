#!/bin/bash -x
cd /rust/examples/daml2oas
cargo build --release --target x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/daml2oas
