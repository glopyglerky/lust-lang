#!/usr/bin/env bash
# rustc baseline: cold check, cold debug build, and one-line-edit incremental check/build.
# Usage: bench.sh <shape: plain|idiomatic> <units>. About 16 to 19 lines per unit.
set -euo pipefail
shape=$1; units=$2
here=$(cd "$(dirname "$0")" && pwd)
dir=${TMPDIR:-/tmp}/lust-bench/run-$shape-$units
rm -rf "$dir"; mkdir -p "$dir"; cd "$dir"
python3 "$here/gen.py" "$shape" "$units" 0 > main.rs
lines=$(wc -l < main.rs)
flags=(--edition 2024 -C panic=abort -A dead_code -A unused)
t() { local s=$(date +%s.%N); "$@" >/dev/null 2>err.txt || { cat err.txt | head -20; exit 1; }; echo "$(date +%s.%N) - $s" | bc; }

check_cold=$(t rustc "${flags[@]}" --emit=metadata -C incremental=inc-check -o lib.rmeta main.rs)
build_cold=$(t rustc "${flags[@]}" -C debuginfo=2 -C incremental=inc-build -o app main.rs)
python3 "$here/gen.py" "$shape" "$units" 1 > main.rs
check_edit=$(t rustc "${flags[@]}" --emit=metadata -C incremental=inc-check -o lib.rmeta main.rs)
build_edit=$(t rustc "${flags[@]}" -C debuginfo=2 -C incremental=inc-build -o app main.rs)
printf "%-10s %6s lines | check cold %6.2fs | check edit %6.2fs | build cold %6.2fs | build edit %6.2fs\n" \
  "$shape" "$lines" "$check_cold" "$check_edit" "$build_cold" "$build_edit"
