#!/usr/bin/env bash
# Differential gate for the erased hash core (src/runtime/hash.c) and the
# IndexMap core in it. A Rust oracle (rustc 1.99, rustc-hash 2.1.2,
# indexmap 2.14.0, hashbrown 0.17.1) and the C harness (pinned tcc) run
# the same random op streams and print len, capacity, and full
# iteration order after every op; outputs must match byte for byte.
#
# Usage: tests/hash/test.sh [--full] [--bench]
#   default: OPS=40000 ops per stream, 2 seeds per mode (env OPS/SEEDS).
#   --full:  OPS=160000, so map+index modes pass 1M checked operations.
#   --bench: also run interleaved bench rounds (ns/op).
# Modes: fx64 fx32 fxstr sip64 set64 (map streams), im64 imstr (index
# streams), hb8 (hashbrown HashTable; run twice: width-16 oracle + C,
# and width-8 oracle built without SSE2 + C built with width 8).
set -uo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
ops=${OPS:-40000}
seeds=${SEEDS:-2}
bench=0
for arg in "$@"; do
  case "$arg" in
    --full) ops=160000 ;;
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

echo "building harness (tcc, gcc, tcc width-8) and oracles (cargo, cargo without SSE2)..."
"$tcc" -w "$root/tests/hash/harness.c" -o "$work/harness" || exit 1
"$tcc" -w -DLUST_HASH_GROUP_WIDTH=8 "$root/tests/hash/harness.c" -o "$work/harness_w8" || exit 1
cc -O1 -w "$root/tests/hash/harness.c" -o "$work/harness_gcc" || exit 1
(cd "$root/tests/hash/oracle" && cargo build --release --offline -q) || exit 1
oracle="$root/tests/hash/oracle/target/release/lust-hash-oracle"
(cd "$root/tests/hash/oracle" && RUSTFLAGS="-C target-feature=-sse,-sse2" cargo build --release --offline -q --target-dir target-nosse) || exit 1
oracle8="$root/tests/hash/oracle/target-nosse/release/lust-hash-oracle"

fail=0
echo "directed edges and FxHasher vectors..."
"$work/harness" edges || fail=1
"$work/harness_gcc" edges >/dev/null || fail=1
"$work/harness" fxvectors || fail=1
if cmp -s <("$oracle" fxvals) <("$work/harness" fxvals); then
  echo "  ok: fxvals"
else
  echo "FAIL: fxvals"; diff <("$oracle" fxvals) <("$work/harness" fxvals) | head -10; fail=1
fi

# hash.h's inline lookups (a caller's unit, an optimizing compiler) against hash.c's
# functions (tests/hash/inl.c): both group widths, and on x86_64 the SWAR group at width 16.
echo "inline lookups against the functions..."
inl_variants="w16: w8:-DLUST_HASH_GROUP_WIDTH=8"
[ "$(uname -m)" = x86_64 ] && inl_variants="$inl_variants w16swar:-mno-sse2"
for v in $inl_variants; do
  n=${v%%:*} f=${v#*:}
  if cc -O2 -w $f -DINL_IMPL -c "$root/tests/hash/inl.c" -o "$work/inl-impl-$n.o" &&
    cc -O2 -w $f -c "$root/tests/hash/inl.c" -o "$work/inl-$n.o" &&
    cc "$work/inl-$n.o" "$work/inl-impl-$n.o" -o "$work/inl-$n"; then
    for seed in $(seq 1 "$seeds"); do
      out=$("$work/inl-$n" "$seed" $((ops * 5))) && echo "  $n $out" || { echo "FAIL: inl-$n seed=$seed: $out"; fail=1; }
    done
  else
    echo "FAIL: inl-$n (build)"; fail=1
  fi
done

total_ops=0
check() {
  # name, stream-kind, oracle-bin, harness-bin, mode, seed
  python3 "$root/tests/hash/gen_ops.py" "$2" "$6" "$ops" > "$work/ops.txt"
  if ! "$3" "$5" < "$work/ops.txt" > "$work/oracle.out" 2>"$work/oracle.err"; then
    echo "FAIL: $1 (oracle exited nonzero)"; head -5 "$work/oracle.err"; fail=1; return
  fi
  if ! "$4" "$5" < "$work/ops.txt" > "$work/harness.out" 2>"$work/harness.err"; then
    echo "FAIL: $1 (harness exited nonzero)"; head -5 "$work/harness.err"; fail=1; return
  fi
  if cmp -s "$work/oracle.out" "$work/harness.out"; then
    echo "  ok: $1 seed=$6 ops=$ops"
    total_ops=$((total_ops + ops))
  else
    echo "FAIL: $1 seed=$6"
    diff "$work/oracle.out" "$work/harness.out" | head -14
    fail=1
  fi
}

for seed in $(seq 1 "$seeds"); do
  for mode in fx64 fx32 fxstr sip64 set64; do
    check "$mode" map "$oracle" "$work/harness" "$mode" $((seed * 1000 + 7))
  done
  for mode in im64 imstr; do
    check "$mode" index "$oracle" "$work/harness" "$mode" $((seed * 2000 + 13))
  done
  check hb16 map "$oracle" "$work/harness" hb8 $((seed * 3000 + 21))
  check hb8 map "$oracle8" "$work/harness_w8" hb8 $((seed * 3000 + 21))
done

# The gcc build must agree too (guards against tcc-specific codegen luck).
check fx64-gcc map "$oracle" "$work/harness_gcc" fx64 4242
check im64-gcc index "$oracle" "$work/harness_gcc" im64 4243

echo "checked operations (passed streams only): $total_ops"

if [ "$bench" = 1 ]; then
  echo "bench (3 interleaved rounds; see README for method):"
  for _ in 1 2 3; do
    "$oracle" bench-fx
    "$work/harness" bench-fx
  done
  for _ in 1 2 3; do
    "$oracle" bench-im
    "$work/harness" bench-im
  done
fi

exit "$fail"
