#!/bin/bash
set -e

# Run Rust linters
cargo clippy -- -D warnings
cargo fmt --check

# Run Solidity linter
solhint -c 'crates/proof-of-sql/.solhint.json' 'crates/proof-of-sql/**/*.sol' -w 0