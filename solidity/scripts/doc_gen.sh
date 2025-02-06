#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR/..
scripts/pre_forge.sh doc
cd docs
sed -i '/\[output\.html\]/a mathjax-support = true' book.toml
mdbook "$@"
