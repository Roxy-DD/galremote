#!/bin/bash
set -e
export PATH="/c/msys64/ucrt64/bin:/c/msys64/usr/bin:$PATH"

gcc_version='15.1.0-5'
broken_deps=("mingw-w64-ucrt-x86_64-gcc" "mingw-w64-ucrt-x86_64-gcc-libs")
tarballs=""

for dep in "${broken_deps[@]}"; do
  tarball="${dep}-${gcc_version}-any.pkg.tar.zst"
  wget -q https://repo.msys2.org/mingw/ucrt64/${tarball}
  tarballs="${tarballs} ${tarball}"
done

if [ -n "$tarballs" ]; then
  pacman -U --noconfirm ${tarballs}
fi

dependencies=("git" "mingw-w64-ucrt-x86_64-cmake" "mingw-w64-ucrt-x86_64-ninja" "mingw-w64-ucrt-x86_64-cppwinrt" "mingw-w64-ucrt-x86_64-curl-winssl" "mingw-w64-ucrt-x86_64-graphviz" "mingw-w64-ucrt-x86_64-MinHook" "mingw-w64-ucrt-x86_64-miniupnpc" "mingw-w64-ucrt-x86_64-nlohmann-json" "mingw-w64-ucrt-x86_64-nodejs" "mingw-w64-ucrt-x86_64-nsis" "mingw-w64-ucrt-x86_64-onevpl" "mingw-w64-ucrt-x86_64-openssl" "mingw-w64-ucrt-x86_64-opus" "mingw-w64-ucrt-x86_64-toolchain")
pacman -Syu --noconfirm --ignore="$(IFS=,; echo "${broken_deps[*]}")" "${dependencies[@]}"
