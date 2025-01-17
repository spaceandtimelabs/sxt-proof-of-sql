#!/usr/bin/env bash

# Optional help text
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo "Runs only the cargo commands within the 'examples:' job in lint-and-test.yml,"
  echo "without stopping the shell on errors. Prints a summary of any failures."

fi

# Location of GitHub Actions workflow file
YAML_FILE=".github/workflows/lint-and-test.yml"

# Ensure the file exists
if [ ! -f "$YAML_FILE" ]; then
  echo "Error: $YAML_FILE not found."
  echo "Please run this script from the repository root or update YAML_FILE accordingly."
  echo "Exiting with status 0, no commands executed."

fi

################################################################################
# 1) Extract only the lines from 'examples:' up until the next top-level job
#
#    - We look for lines starting with exactly two spaces, then 'examples:',
#      and capture everything until the next line that also starts with two
#      spaces and ends with a colon (i.e., '  coverage:', '  clippy:', etc.).
#
# 2) Then find lines containing `cargo` commands and strip off 'run:'.
################################################################################
cargo_commands=$(
  sed -n -E '/^  examples:/,/^  [A-Za-z0-9_]+:/p' "$YAML_FILE" \
    | grep -E '^\s*run:.*cargo' \
    | sed -E 's/^\s*run:\s*//'
)

# If we didn't find any commands, no big deal—just exit quietly.
if [ -z "$cargo_commands" ]; then
  echo "No cargo commands were found in the 'examples:' job block."
  echo "Nothing to run."

fi

echo "Commands extracted from the 'examples:' job:"
echo "--------------------------------------------"
echo "$cargo_commands"
echo "--------------------------------------------"
echo

################################################################################
# 3) Run each command, but do NOT use exit codes. We'll collect failures
#    and report at the end.
################################################################################
total=0
failed=0

while IFS= read -r cmd; do
  total=$((total + 1))
  echo "[$total] Running: $cmd"

  if ! eval "$cmd"; then
    echo "    -> Command FAILED (continuing)."
    failed=$((failed + 1))
  else
    echo "    -> Command succeeded."
  fi

  echo
done <<< "$cargo_commands"

################################################################################
# 4) Print a summary of how many commands failed. Always exit 0 to avoid
#    crashing the shell or signaling an error code.
################################################################################
echo "Summary of examples job:"
echo "  Total commands: $total"
echo "  Failed commands: $failed"
if [ "$failed" -gt 0 ]; then
  echo "Some commands failed, but we are NOT exiting with a non-zero status."
else
  echo "All commands completed successfully!"
fi
