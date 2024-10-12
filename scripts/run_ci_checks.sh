#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Display a help text
[ "$1" = "-h" -o "$1" = "--help" ] && echo "Runs all CI checks (excluding tests and udeps)." && exit

# The path to the YAML file that defines the CI workflows
YAML_FILE=".github/workflows/lint-and-test.yml"

# Initialize the directory we're searching from (current directory)
current_dir=$(pwd)

# Traverse upwards to find the root directory, assuming it exists somewhere above
while [[ ! -f "$current_dir/sxt-proof-of-sql/.github/workflows/lint-and-test.yml" ]]; do
  # Move up one directory
  current_dir=$(dirname "$current_dir")
  
  # If we reach the root directory (i.e., /), stop to prevent an infinite loop
  if [[ "$current_dir" == "/" ]]; then
    echo "Could not find file."
    exit 1
  fi
done

# Check if the YAML file exists
if [ ! -f "$YAML_FILE" ]; then
    echo "YAML file $YAML_FILE does not exist."
    exit 1
fi

# Extract all lines that contain 'cargo' commands from the YAML file, 
# excluding ones with '--ignored', 'test', 'rustup', or 'udeps'
cargo_commands=$(grep -E '^\s*run:.*cargo' "$YAML_FILE" | grep -v -- '--ignored' | grep -v 'test' | grep -v 'rustup' | grep -v 'udeps' | sed -E 's/^\s*run:\s*//')

if [ -z "$cargo_commands" ]; then
    echo "No cargo commands (other than tests) found in the YAML file."
    exit 1
fi

# Run each cargo command, ignoring tests which should be handled separately
echo "Extracted cargo commands (excluding test commands, --ignored tests, and udeps):"
echo "$cargo_commands"
echo "========================="

# Execute the commands
while IFS= read -r cmd; do
    echo "Running command: $cmd"
    if ! eval "$cmd"; then
        echo "Error: Command failed - $cmd"
        echo "Stopping execution."
        exit 1
    fi
done <<< "$cargo_commands"

echo "All CI checks (excluding tests and udeps) have completed successfully."