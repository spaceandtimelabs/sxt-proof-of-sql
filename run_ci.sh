#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Display a help text
[ "$1" = "-h" -o "$1" = "--help" ] && echo "Runs all CI checks (excluding tests)." && exit

# The path to the YAML file that defines the CI workflows
YAML_FILE=".github/workflows/lint-and-test.yml"

# Check if the YAML file exists
if [ ! -f "$YAML_FILE" ]; then
    echo "YAML file $YAML_FILE does not exist."
    exit 1
fi

# Extract all lines that contain 'cargo' commands from the YAML file, 
# excluding ones with '--ignored' or 'test'
cargo_commands=$(grep -E '^\s*run:.*cargo' "$YAML_FILE" | grep -v -- '--ignored' | grep -v 'test' | sed -E 's/^\s*run:\s*//')

if [ -z "$cargo_commands" ]; then
    echo "No cargo commands (other than tests) found in the YAML file."
    exit 1
fi

# Run each cargo command, ignoring tests which should be handled separately
echo "Extracted cargo commands (excluding test commands and --ignored tests):"
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

echo "All CI checks (excluding tests) have completed successfully."

