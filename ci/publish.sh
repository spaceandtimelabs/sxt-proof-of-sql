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

cargo publish -p proof-of-sql-parser --dry-run
#cargo publish -p proof-of-sql --dry-run
