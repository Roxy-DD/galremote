#!/bin/bash
set -e

# Setup paths
export PATH="/c/msys64/ucrt64/bin:/c/msys64/usr/bin:$PATH"
WINDOWS_USER=$(cmd //c "echo %USERNAME%" | tr -d '\r')
CARGO_BIN="/c/Users/${WINDOWS_USER}/.cargo/bin"
export PATH="$CARGO_BIN:$PATH"

echo "=== Environment Info ==="
echo "PATH: $PATH"
which cmake
which ninja
which node
which npm
which cargo
cmake --version | head -n 1
ninja --version
cargo --version

echo "=== Starting Build ==="
mkdir -p build
cmake -B build -G Ninja -S . \
  -DBUILD_DOCS=OFF \
  -DSUNSHINE_ASSETS_DIR=assets \
  -DSUNSHINE_PUBLISHER_NAME="${GITHUB_REPOSITORY_OWNER}" \
  -DSUNSHINE_PUBLISHER_WEBSITE="https://github.com/qiin2333/Sunshine-Foundation" \
  -DSUNSHINE_PUBLISHER_ISSUE_URL="https://github.com/qiin2333/Sunshine-Foundation/issues"

echo "=== Running Ninja ==="
ninja -v -C build
ninja -v -C build sunshine-control-panel

echo "=== Build Completed Successfully ==="
