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
# Aggressively remove all SAL annotations from NVAPI SDK headers
# These cause pervasive compilation errors with GCC/MinGW
# 1. Remove __success parameterized annotations
find third-party/nvapi-open-source-sdk -type f \( -name "*.h" -o -name "*.c" -o -name "*.cpp" \) -exec sed -E -i 's/__success\([^)]*\)//g' {} +

# 2. Strip simple SAL annotations (word boundary ensures we don't hit __cdecl etc)
for macro in __in __out __inout __in_opt __out_opt __inout_opt __checkReturn __success; do
    find third-party/nvapi-open-source-sdk -type f \( -name "*.h" -o -name "*.c" -o -name "*.cpp" \) -exec sed -i "s/\b${macro}\b//g" {} +
done

# 3. Strip other parameterized SAL annotations (e.g. __in_ecount(size))
find third-party/nvapi-open-source-sdk -type f \( -name "*.h" -o -name "*.c" -o -name "*.cpp" \) -exec sed -E -i 's/\b__[a-z_]+\([^)]*\)//g' {} +

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
