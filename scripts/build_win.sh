#!/bin/bash
set -ex

# Setup paths
export PATH="/c/msys64/ucrt64/bin:/c/msys64/usr/bin:$PATH"
WINDOWS_USER="${USER:-runneradmin}"
CARGO_BIN="/c/Users/${WINDOWS_USER}/.cargo/bin"
export PATH="$CARGO_BIN:$PATH"

echo "=== Environment Info ==="
echo "PATH: $PATH"
which cmake
which ninja
which node
which npm
which cargo
cmake --version || echo "CMAKE FAILED"
ninja --version || echo "NINJA FAILED"
cargo --version || echo "CARGO FAILED"

echo "=== Patching Submodules for GCC Compatibility ==="
sed -i 's/extern __success(return == NVAPI_OK) NvAPI_Status/extern NvAPI_Status/g' third-party/nvapi-open-source-sdk/nvapi_lite_salstart.h

echo "=== Starting Build ==="
mkdir -p build
cmake -B build -G Ninja -S . \
  -DBUILD_DOCS=OFF \
  -DCMAKE_PREFIX_PATH="C:/msys64/ucrt64" \
  -DOPENSSL_ROOT_DIR="C:/msys64/ucrt64" \
  -DSUNSHINE_ASSETS_DIR=assets \
  -DSUNSHINE_PUBLISHER_NAME="${GITHUB_REPOSITORY_OWNER}" \
  -DSUNSHINE_PUBLISHER_WEBSITE="https://github.com/qiin2333/Sunshine-Foundation" \
  -DSUNSHINE_PUBLISHER_ISSUE_URL="https://github.com/qiin2333/Sunshine-Foundation/issues"

echo "=== Running Ninja ==="
ninja -v -C build
ninja -v -C build sunshine-control-panel

echo "=== Build Completed Successfully ==="
