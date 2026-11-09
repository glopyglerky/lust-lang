#!/usr/bin/env bash
# pgo-llvm.sh: installs the LLVM a PGO release with zig needs, and prints its env lines
# (LUST_PGO_CC=..., LUST_PROFDATA=...) for $GITHUB_ENV or `eval "$(...)"`.
#
# The release zig (0.15.2: its cc is clang 20.1.2) ships no profile runtime, so the
# instrumented build (LUST_PGO_GEN=1) compiles with a clang of zig's LLVM major for the
# host, and the training's raw profiles merge with that LLVM's llvm-profdata. The profile
# format and the fns' CFG hashes are those of one LLVM major; the release driver
# (src/cc/pgo.rs) refuses a clang or llvm-profdata of another major than the compiler's.
# The patch version may differ (apt.llvm.org ships the latest 20.1.x).
#
# The major is zig's (`zig cc --version`) when zig is on PATH, else LLVM_MAJOR, else 20.
# Installs from apt.llvm.org (Debian/Ubuntu, needs sudo and the network) when the clang
# or its profile runtime is missing. Exit 0 = installed or present, with the two lines on
# stdout; 2 = no way to install here.
set -euo pipefail
major=${LLVM_MAJOR:-}
if [ -z "$major" ] && command -v zig > /dev/null; then
  major=$(zig cc --version | sed -n 's/.*clang version \([0-9][0-9]*\)\..*/\1/p' | head -1)
fi
major=${major:-20}
arch=$(uname -m)
rt=/usr/lib/llvm-$major/lib/clang/$major/lib/linux/libclang_rt.profile-$arch.a
if ! command -v "clang-$major" > /dev/null || ! command -v "llvm-profdata-$major" > /dev/null || [ ! -f "$rt" ]; then
  command -v apt-get > /dev/null || { echo "pgo-llvm: no apt-get here; install clang-$major, llvm-profdata-$major and its profile runtime" >&2; exit 2; }
  tmp=$(mktemp -d)
  curl -fsSL https://apt.llvm.org/llvm.sh -o "$tmp/llvm.sh"
  sudo bash "$tmp/llvm.sh" "$major" >&2
  sudo apt-get install -y "libclang-rt-$major-dev" "llvm-$major" >&2
  rm -rf "$tmp"
fi
[ -f "$rt" ] || { echo "pgo-llvm: no profile runtime at $rt" >&2; exit 2; }
echo "pgo-llvm: $("clang-$major" --version | head -1) (zig: $(command -v zig > /dev/null && zig cc --version | head -1 || echo none))" >&2
echo "LUST_PGO_CC=clang-$major"
echo "LUST_PROFDATA=llvm-profdata-$major"
