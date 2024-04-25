#!/bin/bash
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
sed -i 's/path = "[^"]*"/version = "'${NEW_VERSION}'", registry = "artifactory"/g' Cargo.toml

zip -r proofs-v$NEW_VERSION.zip * -x '*.json*' -x '*target*' -x '*ci*' -x '*.gitignore*' -x '*node_modules*' -x '*Cargo.lock*'
tar --exclude='*.json*' --exclude='*target*' --exclude='*ci*' --exclude='*.gitignore*' --exclude='*Cargo.lock*' --exclude='*node_modules*' -czvf proofs-v$NEW_VERSION.tar.gz *