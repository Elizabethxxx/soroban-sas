#!/bin/bash
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/soroban_sas.wasm --source alice --network testnet
cargo build -p soroban-sas-indexer --release --target wasm32-unknown-unknown
