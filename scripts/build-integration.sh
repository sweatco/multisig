#!/bin/bash
set -eox pipefail

echo ">> Building contract"

rustup target add wasm32-unknown-unknown
cargo build -p multisig --target wasm32-unknown-unknown --profile=contract --features integration-test

cp ./target/wasm32-unknown-unknown/contract/multisig.wasm res/multisig.wasm
