#!/bin/bash

# Builds and publishes image to Docker Hub.
# This is intended to be run by our GitHub Actions workflow.
#
# MUST be run from the root of the repository so the Docker build context is correct.
#
# You must `docker login ...` first so that we have the necessary permission to
# push the image layers + tags to Docker Hub.

SNAPCHAIN_VERSION=$(awk -F '"' '/^version =/ {print $2}' Cargo.toml)

echo "Publishing $SNAPCHAIN_VERSION"

depot build -f Dockerfile \
  --platform "linux/amd64,linux/arm64" \
  --push \
  -t farcasterxyz/snapchain:${SNAPCHAIN_VERSION} \
  -t farcasterxyz/snapchain:latest \
  .
