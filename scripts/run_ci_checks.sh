#!/bin/bash

# Display a help text
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo "Runs all CI checks (excluding tests, udeps, and the 'examples' job)."
fi

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
  fi
done

# Check if the YAML file exists
if [ ! -f "$YAML_FILE" ]; then
    echo "YAML file $YAML_FILE does not exist."
fi

# 1) Remove the entire `examples:` job section from the file.
# 2) Extract lines that contain 'cargo' commands.
# 3) Exclude lines with '--ignored', 'test', 'rustup', or 'udeps'.
# 4) Strip off the 'run:' prefix.
cargo_commands=$(
  sed '/^\s*examples:/,/^[^[:space:]]/d' "$YAML_FILE" \
    | grep -E '^\s*run:.*cargo' \
    | grep -v -- '--ignored' \
    | grep -v 'test' \
    | grep -v 'rustup' \
    | grep -v 'udeps' \
    | sed -E 's/^\s*run:\s*//'
)

if [ -z "$cargo_commands" ]; then
    echo "No cargo commands found (other than tests, udeps, or in the 'examples' job)."
fi

# Run each cargo command
echo "Extracted cargo commands (excluding tests, udeps, 'examples' job, and toolchain installs):"
echo "$cargo_commands"
echo "========================="

failed_tests=0
while IFS= read -r cmd; do
    echo "Running command: $cmd"
    if ! eval "$cmd"; then
        echo "Error: Command failed - $cmd"
        echo "Stopping execution."
        failed_tests=$((failed_tests + 1))
    fi
done <<< "$cargo_commands"

# Print the results
if [ "$failed_tests" -gt 0 ]; then
    echo "Error: $failed_tests CI checks have FAILED (excluding tests, udeps, and 'examples' job)."
else
    echo "All CI checks (excluding tests, udeps, and 'examples' job) completed successfully."
fi
