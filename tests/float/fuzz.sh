#!/usr/bin/env bash
# Fuzz gate for the C float runtime (src/runtime/float.c).
# The same generated cases run through a rustc oracle (and the ryu-js
# crate for JS formatting) and the C harness built with the pinned
# tcc; outputs must match byte for byte.
#
# Usage: tests/float/fuzz.sh [--full] [--bench]
#   default COUNT=20000 cases per function (env COUNT overrides).
#   --full  runs 10,000,000 cases per function (1,000,000 spec combos).
#   --bench also prints ns/call for the tcc and gcc builds.
set -uo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
count=${COUNT:-20000}
spec_count=$count
bench=0
for arg in "$@"; do
  case "$arg" in
    --full) count=10000000; spec_count=1000000 ;;
    --bench) bench=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

tcc=${LUST_TCC_BIN:-}
if [ -z "$tcc" ] && [ -n "${LUST_TCC_HOME:-}" ]; then tcc=$LUST_TCC_HOME/bin/tcc; fi
if [ -z "$tcc" ]; then tcc=$root/third_party/tcc/bin/tcc; fi
if [ ! -x "$tcc" ]; then
  echo "pinned tcc not found (set LUST_TCC_HOME or run scripts/build-tcc.sh)" >&2
  exit 1
fi

echo "building harness (tcc), oracle (rustc), gen (cc)..."
"$tcc" -w "$root/tests/float/harness.c" -o "$work/harness" || exit 1
cc -O2 -w "$root/tests/float/harness.c" -o "$work/harness_gcc" || exit 1
cc -O2 -w "$root/tests/float/gen.c" -o "$work/gen" -lm || exit 1
rustc -O --edition 2024 "$root/tests/float/oracle.rs" -o "$work/oracle" || exit 1

# JS oracle: the real ryu-js crate, if its sources are in the cargo
# registry (it is a ts-rust dependency, so normally present).
oracle_js=""
ryu_src=$(ls -d "$HOME"/.cargo/registry/src/*/ryu-js-1.0.2/src 2>/dev/null | head -1)
if [ -n "$ryu_src" ]; then
  if rustc -O --edition 2021 --crate-name ryu_js --crate-type rlib "$ryu_src/lib.rs" -o "$work/libryu_js.rlib" 2>"$work/ryu.err" &&
     rustc -O --edition 2024 --extern ryu_js="$work/libryu_js.rlib" "$root/tests/float/oracle_js.rs" -o "$work/oracle_js" 2>>"$work/ryu.err"; then
    oracle_js="$work/oracle_js"
  else
    echo "WARN: ryu-js oracle failed to build; JS fuzz skipped:" >&2
    head -5 "$work/ryu.err" >&2
  fi
else
  echo "WARN: ryu-js crate sources not found; JS fuzz skipped (edges still run)" >&2
fi

echo "directed edges..."
"$work/harness" edges || exit 1
"$work/harness_gcc" edges || exit 1

fail=0
check() {
  # name, gen mode, count, oracle binary, oracle/harness mode, seed
  echo "fuzz $1 (n=$3, seed=$6)..."
  if diff <("$work/gen" "$2" "$3" "$6" | "$4" "$5") <("$work/gen" "$2" "$3" "$6" | "$work/harness" "$5") >"$work/diff.txt"; then
    echo "  ok: $1"
  else
    echo "FAIL: $1"
    head -20 "$work/diff.txt"
    fail=1
  fi
}

seed=${SEED:-12345}
check fmt_f64     bits64 "$count" "$work/oracle" fmt64    "$seed"
check fmt_f32     bits32 "$count" "$work/oracle" fmt32    "$((seed + 1))"
check spec_f64    spec64 "$spec_count" "$work/oracle" spec64 "$((seed + 2))"
check spec_f32    spec32 "$spec_count" "$work/oracle" spec32 "$((seed + 3))"
check parse_f64   dec    "$count" "$work/oracle" parse64  "$((seed + 4))"
check parse_f32   dec    "$count" "$work/oracle" parse32  "$((seed + 5))"
check casts_f64   bits64 "$count" "$work/oracle" cast64   "$((seed + 6))"
check casts_f32   bits32 "$count" "$work/oracle" cast32   "$((seed + 7))"
check casts_128   bits64 "$count" "$work/oracle" cast128  "$((seed + 8))"
check casts_128f  bits32 "$count" "$work/oracle" cast128f "$((seed + 9))"
check int_casts   ints   "$count" "$work/oracle" intcast  "$((seed + 10))"
if [ -n "$oracle_js" ]; then
  check js_f64 bits64 "$count" "$oracle_js" js64 "$((seed + 11))"
  check js_f32 bits32 "$count" "$oracle_js" js32 "$((seed + 12))"
fi

# Compiler independence: the gcc build must give the same answers on a
# smaller slice (the exact algorithms should not care who compiled them).
slice=$count
if [ "$slice" -gt 100000 ]; then slice=100000; fi
echo "gcc slice (n=$slice)..."
harness_save="$work/harness"
cp "$work/harness_gcc" "$work/harness_gcc_run"
for m in fmt64 parse64 cast64; do
  case $m in
    fmt64) g=bits64 ;;
    parse64) g=dec ;;
    cast64) g=bits64 ;;
  esac
  if diff <("$work/gen" "$g" "$slice" 777 | "$work/oracle" "$m") <("$work/gen" "$g" "$slice" 777 | "$work/harness_gcc_run" "$m") >"$work/diff.txt"; then
    echo "  ok: gcc $m"
  else
    echo "FAIL: gcc $m"
    head -20 "$work/diff.txt"
    fail=1
  fi
done
: "$harness_save"

if [ "$bench" = 1 ]; then
  echo "bench (tcc build):"
  "$work/harness" bench
  echo "bench (gcc -O2 build):"
  "$work/harness_gcc" bench
fi

if [ "$fail" != 0 ]; then
  echo "float fuzz: FAILED"
  exit 1
fi
echo "float fuzz: all ok (count=$count, spec_count=$spec_count)"
