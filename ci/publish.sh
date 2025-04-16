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
  # Make sure to use the correct index URL for crates.io since local crates are otherwise considered
  # which will always succeed and nothing will be published
  if cargo info --index https://github.com/rust-lang/crates.io-index \
     "${crate}@${NEW_VERSION}" >/dev/null 2>&1
  then
    echo "The version ${NEW_VERSION} for ${crate} is already on crates.io. Skipping publish."
  else
    echo "${crate}@${NEW_VERSION} not found, publishing..."
    cargo publish -p "${crate}" --token "${CRATES_TOKEN}"
  fi
done
