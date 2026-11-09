#!/usr/bin/env bash
# Differential gate for the erased Vec core and the alloc family in
# src/runtime/core.c (ARCH 3.3). A rustc 1.99 oracle and the C harness
# (pinned tcc, plus cc as a guard against tcc-specific luck) run the
# same capacity sequences; outputs must match byte for byte.
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

rustc -O -Awarnings "$root/tests/vec/oracle.rs" -o "$work/oracle" || exit 1
"$tcc" -w "$root/tests/vec/harness.c" -o "$work/harness" || exit 1
cc -O1 -w "$root/tests/vec/harness.c" -o "$work/harness_gcc" || exit 1

fail=0
for h in harness harness_gcc; do
  if cmp -s <("$work/oracle") <("$work/$h"); then
    echo "  ok: capacity sequences ($h)"
  else
    echo "FAIL: capacity sequences ($h)"
    diff <("$work/oracle") <("$work/$h") | head -20
    fail=1
  fi
  if "$work/$h" alloc > "$work/alloc.out" 2>&1 && grep -q "alloc ok" "$work/alloc.out"; then
    echo "  ok: aligned alloc ($h)"
  else
    echo "FAIL: aligned alloc ($h)"; cat "$work/alloc.out"; fail=1
  fi
done
exit "$fail"
