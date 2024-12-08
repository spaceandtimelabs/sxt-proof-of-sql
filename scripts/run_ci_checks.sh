#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Display help text
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    echo "Usage: $0 [-h|--help]"
    echo "Description: This script runs all CI checks excluding tests and udeps."
    exit 0
fi

# Initialize directory and file paths
current_dir=$(realpath $(dirname "$0"))
root_dir=$(find "$current_dir" -type d -name "sxt-proof-of-sql" -print -quit)
if [ -z "$root_dir" ]; then
    echo "Could not find root directory."
    exit 1
fi

YAML_FILE="$root_dir/.github/workflows/lint-and-test.yml"

# Check if the YAML file exists
if [ ! -f "$YAML_FILE" ]; then
    echo "YAML file $YAML_FILE does not exist."
    exit 1
fi

# Extract all relevant 'cargo' commands from the YAML file
exclude_patterns="--ignored|test|rustup|udeps"
cargo_commands=$(grep -E '^\s*run:.*cargo' "$YAML_FILE" | grep -vE "$exclude_patterns" | sed -E 's/^\s*run:\s*//')

if [ -z "$cargo_commands" ]; then
    echo "No cargo commands (other than tests) found in the YAML file."
    exit 1
fi

# Display and execute extracted commands
echo "Extracted cargo commands (excluding test commands, --ignored tests, and udeps):"
echo "$cargo_commands"
echo "========================="

failed_tests=0

while IFS= read -r cmd; do
    echo "Running command: $cmd"
    if ! bash -c "$cmd"; then
        echo "Error: Command failed - $cmd"
        echo "Stopping execution."
        failed_tests=$((failed_tests + 1))
    fi
done <<< "$cargo_commands"

# Print summary
if [ "$failed_tests" -gt 0 ]; then
    echo "Error: $failed_tests CI checks (excluding tests and udeps) have FAILED."
    exit 1
else
    echo "All CI checks (excluding tests and udeps) have completed successfully."
fi
