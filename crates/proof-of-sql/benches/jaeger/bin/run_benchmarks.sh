#!/bin/bash

# Ensure the script runs in its current directory
cd "$(dirname "$0")"

# Create a "data" directory if it doesn't already exist
mkdir -p data

# Get the current timestamp in the format "YYYY-MM-DD_HH-MM-SS"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Export the CSV_PATH environment variable
export CSV_PATH="$(pwd)/data/results_${TIMESTAMP}.csv"

# Define the schemes and table sizes to iterate over
SCHEMES=("hyper-kzg", "dynamic-dory")
TABLE_SIZES=(2097152 4194304 8388608 16777216 33554432 67108864)

# Loop over each scheme and table size
for TABLE_SIZE in "${TABLE_SIZES[@]}"; do
    echo "Running benchmark for scheme: $SCHEME, table size: $TABLE_SIZE"
    cargo run --release --bin jaeger_benches --features="bench" -- -s hyper-kzg -i 3 -t "$TABLE_SIZE"
done

cargo run --release --bin jaeger_benches --features="bench" -- -s hyper-kzg -i 3 -t 1048576 -c "$(pwd)/data/"
