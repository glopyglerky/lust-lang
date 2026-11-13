#!/usr/bin/env bash
# Differential gate for the 128-bit integer helpers in
# src/runtime/core.c. The rustc 1.99 oracle and the C harness (pinned
# tcc, plus cc) print wrapping ops, division, shifts, compares, and
# decimal formatting over an edge grid and 12000 random pairs;
# outputs must match byte for byte. Checked ops run one case per
# process: the oracle prints the value or OVERFLOW, and the harness
# must print the value or die with the Rust panic text.
set -uo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

tcc=${LUST_TCC_BIN:-}
if [ -z "$tcc" ] && [ -n "${LUST_TCC_HOME:-}" ]; then tcc=$LUST_TCC_HOME/bin/tcc; fi
if [ -z "$tcc" ]; then tcc=$root/third_party/tcc/bin/tcc; fi
if [ ! -x "$tcc" ]; then
  echo "pinned tcc not found (set LUST_TCC_HOME or run scripts/build-tcc.sh)" >&2
  exit 1
fi

rustc -O -Awarnings "$root/tests/int128/oracle.rs" -o "$work/oracle" || exit 1
"$tcc" -w "$root/tests/int128/harness.c" -o "$work/harness" || exit 1
cc -O1 -w "$root/tests/int128/harness.c" -o "$work/harness_gcc" || exit 1

fail=0
"$work/oracle" > "$work/oracle.out"
for h in harness harness_gcc; do
  if "$work/$h" > "$work/$h.out" && cmp -s "$work/oracle.out" "$work/$h.out"; then
    echo "  ok: wrapping ops and formatting ($h, $(wc -l < "$work/$h.out") lines)"
  else
    echo "FAIL: wrapping ops and formatting ($h)"
    diff "$work/oracle.out" "$work/$h.out" | head -20
    fail=1
  fi
done

# Checked ops: op, a_lo, a_hi, b_lo, b_hi. The overflow text must name
# the operation the way Rust does.
declare -A OVF=(
  [uadd]="add" [usub]="subtract" [umul]="multiply" [iadd]="add"
  [isub]="subtract" [imul]="multiply" [ineg]="negate" [iabs]="negate"
  [upow]="multiply" [ipow]="multiply" [idiv]="divide" [irem]="calculate the remainder"
)
MAX=18446744073709551615
MINHI=9223372036854775808
cases=(
  "uadd $MAX $MAX 1 0" "uadd 40 0 2 0" "usub 0 0 1 0" "umul $MAX $MAX 2 0"
  "umul 3 0 0 1" "udiv 7 0 0 0" "urem 7 0 0 0" "udiv 100 5 7 0"
  "iadd $MAX 9223372036854775807 1 0" "isub 0 $MINHI 1 0"
  "imul 0 $MINHI $MAX $MAX" "imul 6 0 $MAX $MAX" "imul 0 1 0 1"
  "ineg 0 $MINHI 0 0" "iabs 0 $MINHI 0 0" "iabs $MAX $MAX 0 0"
  "idiv 0 $MINHI $MAX $MAX" "irem 0 $MINHI $MAX $MAX"
  "upow 2 0 127 0" "upow 2 0 128 0" "ipow $MAX $MAX 3 0" "ipow 10 0 38 0"
)
for c in "${cases[@]}"; do
  want=$("$work/oracle" checked $c)
  "$work/harness" checked $c > "$work/got.out" 2> "$work/got.err"
  rc=$?
  if [ "$want" != "OVERFLOW" ]; then
    if [ $rc -eq 0 ] && [ "$(cat "$work/got.out")" = "$want" ]; then
      echo "  ok: checked $c -> $want"
    else
      echo "FAIL: checked $c want $want got rc=$rc out=$(cat "$work/got.out") err=$(head -2 "$work/got.err")"
      fail=1
    fi
  else
    op=${c%% *}
    word=${OVF[$op]:-}
    if [ "$op" = "udiv" ] || [ "$op" = "urem" ]; then word="zero"; fi
    if [ $rc -ne 0 ] && grep -q "attempt to" "$work/got.err" && grep -qi "$word" "$work/got.err"; then
      echo "  ok: checked $c -> panic ($word)"
    else
      echo "FAIL: checked $c want panic ($word) got rc=$rc out=$(cat "$work/got.out") err=$(head -2 "$work/got.err")"
      fail=1
    fi
  fi
done
exit "$fail"
