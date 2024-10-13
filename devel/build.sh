#!/bin/bash

set -e

CACHE_ARGS=(--cache-from=nextcloud-atomic-builder)
if [[ "$1" == "release" ]]
then
  echo "Building release image"
  BUILD_TARGET="release"
  IMAGE_NAME="thecalcaholic/ncatomic-activation:embedded"
  CACHE_ARGS+=(--cache-from=nextcloud-atomic-activation-builder)
elif [[ "$1" == "devcontainer" ]]
then
  echo "Building devcontainer"
  BUILD_TARGET="devcontainer"
  IMAGE_NAME="thecalcaholic/ncatomic-devcontainer:embedded"
else
  echo "Invalid build target: $1 (valid: 'devcontainer', 'release')"
  exit 1
fi

(
  cd "$(dirname "$0")" || {
    echo "Failed to go to devel directory '$(dirname "$0")'"
    exit 1
  }
  docker build -t nextcloud-atomic-builder --target=builder -f ./Dockerfile ..
  [[ "$BUILD_TARGET" == "devcontainer" ]] || docker build -t nextcloud-atomic-activation-builder --target=app-builder -f ./Dockerfile ..

  docker build -t "${IMAGE_NAME}" --target="${BUILD_TARGET}" "${CACHE_ARGS[@]}" -f ./Dockerfile ..
)

