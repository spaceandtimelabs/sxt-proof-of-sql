#!/usr/bin/env bash
# This script is used to lint and test the Solidity codebase.
# It is not used in the CI pipeline, but shold be identical to the CI pipeline's test job.
# The CI pipeline is explicitly written out to make the pipeline more readable.
set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR/..
scripts/install_deps.sh
scripts/pre_forge.sh test --summary
scripts/check_coverage.sh
solhint 'src/**/*.sol' 'test/**/*.sol' -w 0
slither .
