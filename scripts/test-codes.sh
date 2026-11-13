#!/usr/bin/env bash
# Self-tests for scripts/check-codes.sh: the real tree must pass, and three
# broken copies in a temp dir must fail (out-of-range code, overlapping
# registry ranges, emitting a reserved code). If a release binary exists,
# also checks `lust explain` answers for a reserved code.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
check=$root/scripts/check-codes.sh
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
pass=0
fail=0
ok() { pass=$((pass + 1)); echo "ok $1"; }
bad() { fail=$((fail + 1)); echo "FAIL $1: $2"; }

# Copies the parts check-codes reads (src/, docs/errors/) into $work/$1.
copy_tree() {
  mkdir -p "$work/$1/docs"
  cp -r "$root/src" "$work/$1/src"
  cp -r "$root/docs/errors" "$work/$1/docs/errors"
}

# 1. The real tree passes.
if "$check" "$root" > "$work/positive.log" 2>&1; then ok positive; else bad positive "$(cat "$work/positive.log")"; fi

# 2. A fake out-of-range code in src/ fails.
copy_tree out-of-range
echo '// fake diagnostic L0999' >> "$work/out-of-range/src/diag.rs"
if "$check" "$work/out-of-range" > "$work/out-of-range.log" 2>&1; then
  bad out-of-range "check-codes passed with a fake L0999 in src/"
elif grep -q 'outside every range' "$work/out-of-range.log"; then ok out-of-range
else bad out-of-range "failed, but not for the range reason: $(cat "$work/out-of-range.log")"; fi

# 3. A fake overlapping registry range fails.
copy_tree collision
echo '| L0305-L0315 | Fake area | test | reserved | |' >> "$work/collision/docs/errors/REGISTRY.md"
if "$check" "$work/collision" > "$work/collision.log" 2>&1; then
  bad collision "check-codes passed with overlapping ranges"
elif grep -q 'overlap' "$work/collision.log"; then ok collision
else bad collision "failed, but not for the overlap reason: $(cat "$work/collision.log")"; fi

# 4. Emitting a code the registry still calls reserved fails.
copy_tree reserved-emit
echo '// fake diagnostic L0501' >> "$work/reserved-emit/src/diag.rs"
if "$check" "$work/reserved-emit" > "$work/reserved-emit.log" 2>&1; then
  bad reserved-emit "check-codes passed while src/ emits reserved L0501"
elif grep -q 'says reserved' "$work/reserved-emit.log"; then ok reserved-emit
else bad reserved-emit "failed, but not for the reserved reason: $(cat "$work/reserved-emit.log")"; fi

# 5. `lust explain` knows the reserved ranges (needs a built binary).
lust=$root/target/release/lust
if [ -x "$lust" ]; then
  out=$("$lust" explain L0501 2>&1)
  if [ $? -eq 0 ] && grep -q 'reserved, not yet emitted' <<<"$out"; then ok explain-reserved
  else bad explain-reserved "lust explain L0501 gave: $out"; fi
  out=$("$lust" explain 2>&1)
  if grep -q 'reserved (not yet emitted):' <<<"$out" && grep -q 'L0501-L0504' <<<"$out"; then ok explain-list
  else bad explain-list "lust explain does not list the reserved ranges"; fi
  out=$("$lust" explain L0999 2>&1)
  if [ $? -eq 2 ] && grep -q 'unknown error code' <<<"$out"; then ok explain-unknown
  else bad explain-unknown "lust explain L0999 gave: $out"; fi
else
  echo "skip explain checks: no binary at $lust"
fi

echo "test-codes: $pass passed, $fail failed"
[ $fail -eq 0 ]
