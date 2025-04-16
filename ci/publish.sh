#!/usr/bin/env bash
set -Eeuxo pipefail

# number to be used to tag the compressed files
NEW_VERSION=$1

if ! [[ ${NEW_VERSION} =~ ^[0-9]+[.][0-9]+[.][0-9]+$ ]]
then
    echo "Incorrect version format: " $NEW_VERSION
    exit 1
fi

# configure rust lib to release
sed -i 's/version = "*.*.*" # DO NOT CHANGE THIS LINE! This will be automatically updated/version = "'${NEW_VERSION}'"/' Cargo.toml
sed -i 's/path = "[^"]*"/version = "'${NEW_VERSION}'"/g' Cargo.toml

CRATES=("proof-of-sql-parser" "proof-of-sql" "proof-of-sql-planner")

for crate in "${CRATES[@]}"; do
  echo "Attempting to see if ${crate}@${NEW_VERSION} is published already..." 
  cargo info "${crate}@${NEW_VERSION}"
  if [ $? -eq 0 ]; then
    echo "The version ${NEW_VERSION} for ${crate} is already on crates.io. Skipping publish."
  else
    echo "${crate}@${NEW_VERSION} not found, publishing..."
    cargo publish -p "${crate}" --token "${CRATES_TOKEN}"
  fi
done
