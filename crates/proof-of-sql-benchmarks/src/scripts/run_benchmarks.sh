#!/bin/bash

# Go to the parent directory of the script
current_folder_folder=$(dirname "$0")
cd "$current_folder_folder/.."

# Create a data folder if one does not exist
mkdir -p "data"

# Get the current timestamp
timestamp=$(date +"%Y%m%d_%H%M%S")

# Define the output CSV file
output_csv="data/results_${timestamp}.csv"

# Define the schemas to benchmark
schemas=("hyper-kzg")

# Define the table sizes to test
table_sizes=(33554432 67108864 134217728 268435456)

# Run benchmarks for each schema and table size
for schema in "${schemas[@]}"; do
    for table_size in "${table_sizes[@]}"; do
        echo "Running benchmark for schema: $schema, table size: $table_size"
        
        # Run the benchmark command
        # Replace `cargo run` with your actual benchmark command
        cargo run --release --bin proof-of-sql-benchmarks -- \
            --scheme "$schema" \
            --table-size "$table_size" \
            --csv-path "$output_csv" \
            --iterations 1
    done
done

echo "Benchmarks completed. Results saved to $output_csv"
