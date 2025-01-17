#!/usr/bin/env bash

if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo "Runs only the 'test' job cargo commands from lint-and-test.yml, ignoring rustup lines."
fi

YAML_FILE=".github/workflows/lint-and-test.yml"

# Ensure the YAML file exists
if [ ! -f "$YAML_FILE" ]; then
  echo "Error: '$YAML_FILE' does not exist."
  echo "No commands to run. Exiting."
fi

###############################################################################
# 1) Extract lines between:
#      ^  test:         (2 spaces + 'test:')
#   and
#      ^  [A-Za-z0-9_]+:   (2 spaces + something else + ':')
#   This ensures we only capture lines up to the next job at the same indentation.
#
# 2) Within that block, look for lines containing `run: cargo ...`.
# 3) Exclude any with 'rustup'.
# 4) Strip off 'run:' prefix.
###############################################################################
cargo_commands=$(
  sed -n '/^  test:/,/^  [A-Za-z0-9_]\+:/p' "$YAML_FILE" \
    | grep -E '^\s*run:.*cargo' \
    | grep -v 'rustup' \
    | sed -E 's/^\s*run:\s*//'
)

if [ -z "$cargo_commands" ]; then
  echo "No cargo commands found in the 'test' job."
  exit 0
fi

echo "Extracted cargo commands from the 'test:' job (skipping 'rustup'):"
echo "------------------------------------------------------------------"
echo "$cargo_commands"
echo "------------------------------------------------------------------"
echo

###############################################################################
# 2) Run each command, counting failures but NOT exiting on them.
###############################################################################
count=0
failed=0

while IFS= read -r cmd; do
  count=$((count + 1))
  echo "[$count] Running command: $cmd"
  
  if ! eval "$cmd"; then
    echo "    -> Command FAILED (continuing)."
    failed=$((failed + 1))
  else
    echo "    -> Command succeeded."
  fi
  echo
done <<< "$cargo_commands"

###############################################################################
# 3) Print a summary. Always exit code 0 (no crash).
###############################################################################
echo "Summary of the 'test' job cargo commands:"
echo "  Total commands: $count"
echo "  Failed commands: $failed"

if [ "$failed" -gt 0 ]; then
  echo "Some commands failed, but we are NOT exiting with a non-zero code."
else
  echo "All commands in the 'test' job completed successfully!"
fi


