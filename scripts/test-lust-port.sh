#!/usr/bin/env bash
# Tests for tools/lust-port. That crate has its own workspace on purpose
# (the root crate must not become a workspace), so the root `cargo test`
# never runs it. Usage: scripts/test-lust-port.sh
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
if cargo test --manifest-path "$root/tools/lust-port/Cargo.toml" >"$work/out" 2>&1; then rc=0; else rc=$?; fi
cat "$work/out"
pass=$(grep -oE '[0-9]+ passed' "$work/out" | awk '{s += $1} END {print s + 0}')
fail=$(grep -oE '[0-9]+ failed' "$work/out" | awk '{s += $1} END {print s + 0}')
if [ "$rc" -ne 0 ] && [ "$fail" -eq 0 ]; then fail=1; fi
echo "test-lust-port: $pass passed, $fail failed"
[ "$rc" -eq 0 ] && [ "$fail" -eq 0 ]
