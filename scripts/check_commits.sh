#!/usr/bin/env bash

CONVENTIONAL_REGEX="^(feat|fix|chore|docs|style|refactor|perf|test|build|ci|revert)(\(.+\))?: .+$"
COMMITS=$(git log origin/main..HEAD --pretty=format:"%s")

count=0
failed=0

while IFS= read -r COMMIT_MSG; do
  count=$((count + 1))
  echo "[$count] Checking commit message: $COMMIT_MSG"
  if [[ ! $COMMIT_MSG =~ $CONVENTIONAL_REGEX ]]; then
    echo "    -> Does NOT match conventional commit format"
    failed=$((failed + 1))
  else
    echo "    -> Matches conventional commit format"
  fi
  echo
done <<< "$COMMITS"

echo "Summary of the conventional commit check:"
echo "  Total commits: $count"
echo "  Failed commits: $failed"

if [ "$failed" -gt 0 ]; then
  echo "Some commits failed the check. Make sure your commit messages match the conventional commit format. 
  Check https://www.conventionalcommits.org/en/v1.0.0/#summary for more details."
else
  echo "All commits match the conventional commit format!"
fi
