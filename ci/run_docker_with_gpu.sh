#!/bin/bash

set -e

DOCKER_REGISTRY="spaceandtime.jfrog.io"
DOCKER_REPO="$DOCKER_REGISTRY/sxt-proofs-dev-docker-local"
# To authenticate to artifactory, you need to uncomment the following line:
# echo "$ARTIFACTORY_PASSWORD" | docker login $DOCKER_REGISTRY -u "$ARTIFACTORY_USER" --password-stdin

IMAGE=$DOCKER_REPO/rust-dev:1.77.1.0

# If you have a GPU instance configured in your machine
docker run --rm -v "$PWD":/proofs -w /proofs --gpus all --privileged -it "$IMAGE"
