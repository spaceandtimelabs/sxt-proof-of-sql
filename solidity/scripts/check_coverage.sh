#!/usr/bin/env bash
set -euo pipefail
solidity/scripts/pre_forge.sh coverage
if [ $(solidity/scripts/pre_forge.sh coverage | grep -o "[0-9\.]*%" | uniq | tr -d '\n') != "%100.00%" ]; then
    >&2 echo "missing test coverage!"
    exit 1
fi
echo "100% test coverage!"
