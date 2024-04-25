#!/bin/bash
set -Eeuxo pipefail

#########################################
# Environment variables:
#########################################
# - GH_TOKEN: token with read access to https://github.com/spaceandtimelabs/proofs

git config --global url."https://api:$GH_TOKEN@github.com/".insteadOf "https://github.com/"
git config --global url."https://ssh:$GH_TOKEN@github.com/".insteadOf "ssh://git@github.com/"
git config --global url."https://git:$GH_TOKEN@github.com/".insteadOf "git@github.com:"
