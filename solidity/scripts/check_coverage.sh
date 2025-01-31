#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR/..
scripts/pre_forge.sh coverage -q --report lcov
gawk -i inplace '
    BEGIN { e=0; p="" }
    $0 ~ /exclude_coverage_start/ { e=1; p=""; next }
    $0 ~ /exclude_coverage_stop/ { e=0; next }
    e==0 { if(p) print p; p=$0 }
    END { if(p) print p }
    ' lcov.info
percentage=$(genhtml lcov.info -o coverage-report --branch-coverage | grep -o "[0-9\.]*%" | uniq | tr -d '\n')
if [ $percentage != "100.0%" ]; then
    >&2 echo "missing test coverage!"
    exit 1
fi
echo "100% test coverage!"
