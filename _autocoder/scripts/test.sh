#!/bin/bash
set -e

# Run Rust tests with all features
cargo test --all-features

# Run Solidity tests
cd crates/proof-of-sql
forge test --summary --detailed