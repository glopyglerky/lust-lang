#!/usr/bin/env bash
# Builds the pinned tinycc (mob) into third_party/tcc. The lust build links its libtcc.a.
set -euo pipefail
TCC_COMMIT=43c7708
root=$(cd "$(dirname "$0")/.." && pwd)
src=$root/third_party/tinycc
out=$root/third_party/tcc
if [ ! -d "$src/.git" ]; then
  git clone -q https://github.com/TinyCC/tinycc.git "$src"
fi
git -C "$src" fetch -q --depth 1 origin "$TCC_COMMIT" 2>/dev/null || true
git -C "$src" checkout -q "$TCC_COMMIT"
cd "$src"
./configure --prefix="$out" >/dev/null
make -j"$(getconf _NPROCESSORS_ONLN)" >/dev/null
make install >/dev/null
echo "tcc $TCC_COMMIT installed in $out"
