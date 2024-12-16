#!/bin/bash
set -e

# Build Rust project with all features
cargo build --all-features

# Build Solidity contracts with Foundry
cd crates/proof-of-sql
forge build