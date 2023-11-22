#!/bin/bash
set -eox pipefail

echo ">> Building contract"

rustup target add wasm32-unknown-unknown
cargo build -p contract_name --target wasm32-unknown-unknown --profile=contract --features integration-test

cp ./target/wasm32-unknown-unknown/contract/contract_name.wasm res/contract_name.wasm
