#!/usr/bin/env bash
# Gate step: the feature statuses committed in tools/lust-port/features.json
# must equal the statuses the feature probes derive on this tree
# (`python3 scripts/port-scan-sync.py --check`). A branch that ran --write
# on an old base can merge a stale status into next and hide real coverage
# (core.std_basics, ext.memchr). This step makes that a red gate instead.
#
# Statuses come only from the probes in tools/lust-port/tests/features/,
# so this step needs no ts-rust checkout. It does need rustc, a built lust
# (target/release/lust or $LUST), and, for the cargo-dep probes, Cargo with
# the offline registry cache (the gate sandbox snapshot has it).
# PORT_SYNC_JOBS sets the probe worker count (default: the script's auto).
# On a status mismatch, the check prints the exact fix: rebase on next,
# rebuild, rerun --write, and commit features.json. A probe that times
# out under load is retried once, serially, with a long limit, and is
# never read as a reject; a probe that times out again fails as a
# TIMEOUT, not a MISMATCH (see port-scan-sync.py; PORT_SCAN_*_TIMEOUT
# env vars set the limits and pass through this script).
# Usage: scripts/port-sync-check.sh
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
lust=${LUST:-$root/target/release/lust}
case $lust in
  /*) ;;
  */*) lust=$root/$lust ;;
esac
if [ ! -x "$lust" ]; then
  echo "error: $lust not built. Run cargo build --release first."
  echo "port-sync-check: 0 passed, 1 failed"
  exit 1
fi
export LUST=$lust
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
jobs=()
[ -n "${PORT_SYNC_JOBS:-}" ] && jobs=(--jobs "$PORT_SYNC_JOBS")
if python3 scripts/port-scan-sync.py --check "${jobs[@]}" >"$work/out" 2>&1; then rc=0; else rc=$?; fi
cat "$work/out"
summary=$(grep -E '^port-scan-sync check: [0-9]+ passed, [0-9]+ failed, [0-9]+ unchecked$' "$work/out" | tail -1)
if [ -n "$summary" ]; then
  pass=$(echo "$summary" | sed -E 's/.*check: ([0-9]+) passed.*/\1/')
  fail=$(echo "$summary" | sed -E 's/.*, ([0-9]+) failed,.*/\1/')
else
  pass=0
  fail=1
fi
# A crash before the summary line, or a bad-probe exit with no feature
# mismatch, is still a failure here.
if [ "$rc" -ne 0 ] && [ "$fail" -eq 0 ]; then fail=1; fi
echo "port-sync-check: $pass passed, $fail failed"
[ "$rc" -eq 0 ] && [ "$fail" -eq 0 ]
