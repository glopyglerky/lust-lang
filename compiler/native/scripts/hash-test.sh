#!/usr/bin/env bash
# The differential gate for the erased hash core (tests/hash/test.sh),
# wrapped for the landing gate (script_step): same run, with the
# checked-operations count and failure count on the last line.
# Passes its arguments through (for example --full).
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
out=$("$root/tests/hash/test.sh" "$@" 2>&1)
rc=$?
echo "$out"
ops=$(echo "$out" | sed -n 's|^checked operations (passed streams only): \([0-9]*\)$|\1|p' | tail -1)
echo "hash-test: ${ops:-0} checked operations, $rc failed"
exit "$rc"
