#!/usr/bin/env bash
# itempatch-sweep.sh [suite ...]: the patch oracle sweep (sigedit PLAN, "Tests and sweeps for
# every slice"). Each suite runs with LUST_CHECK_ITEMPATCH=1: after every patched request the
# daemon checks the warm state against a cold resolve and collect (src/driver/item_oracle.rs),
# after every request the reused bodies against the program, and after every check request its
# answer against a fresh session's. An oracle failure is answered as an internal error (code 3),
# never retried. A suite fails on any line but `itempatch oracle: ok ...` in its oracle log, and a
# daemon suite fails when its log has no `ok` line (the oracle saw none of its edits). difftest
# and crates compile cold only (no daemon): they run as the rule-zero net, with no oracle count.
# With LUST_CHECK_RESOLVE=1 set as well, the resolve oracle (src/driver/resolve_oracle.rs) checks
# every request's resolver state against a cold resolve and logs `resolve oracle: ...` lines to
# the same logs.
#
# Suites (default: all but tsedit):
#   difftest crates driver scenarios   scripts/difftest.sh, test-crates.sh, test-driver.sh,
#                                      test-daemon-scenarios.py
#   multi multi2 multi3 realish        fuzz.py run --sig-ops --rustc, SWEEP_EDITS edits (1500), seed 7
#                                      (realish: 42)
#   patchseq                           the same, once per seed in SWEEP_SEEDS ("7 42 1234")
#   derive                             derive_sweep.py, 100 programs on the chunk path and 100 with
#                                      --resolver
#   tsedit                             scripts/tsrust-full.sh --tsedit (needs the fork; see there),
#                                      plus TSEDIT_ARGS
# LUST_BASELINE=<lust>: the pinned baseline lust for the fuzz suites' rustc judging (sigops.py).
# Logs: $SWEEP_OUT (default $TMPDIR/itempatch-sweep)/<suite>.log and <suite>.oracle.
# Last line per suite: `sweep: <suite> rc=<rc> oracle_ok=<n> oracle_fail=<m>`; exit 1 if any
# suite failed.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
out=${SWEEP_OUT:-${TMPDIR:-/tmp}/itempatch-sweep}
mkdir -p "$out"
edits=${SWEEP_EDITS:-1500}
seeds=${SWEEP_SEEDS:-7 42 1234}
export LUST_CHECK_ITEMPATCH=1
base=()
[ -n "${LUST_BASELINE:-}" ] && base=(--baseline "$LUST_BASELINE")
suites=("$@")
[ ${#suites[@]} -gt 0 ] || suites=(difftest crates driver scenarios multi multi2 multi3 realish patchseq derive)
bad=0
for s in "${suites[@]}"; do
  log=$out/$s.log
  export LUST_ITEMPATCH_LOG=$out/$s.oracle LUST_RESOLVE_LOG=$out/$s.oracle
  rm -f "$LUST_ITEMPATCH_LOG"
  fz=(python3 scripts/fuzz/fuzz.py run --edits "$edits" --sig-ops --rustc "${base[@]}" --out "$out/fuzz-$s")
  case $s in
    difftest) scripts/difftest.sh > "$log" 2>&1 ;;
    crates) scripts/test-crates.sh > "$log" 2>&1 ;;
    driver) scripts/test-driver.sh > "$log" 2>&1 ;;
    scenarios) python3 scripts/test-daemon-scenarios.py > "$log" 2>&1 ;;
    multi|multi2|multi3) "${fz[@]}" --seed 7 --crate "$s" > "$log" 2>&1 ;;
    realish) "${fz[@]}" --seed 42 --crate realish > "$log" 2>&1 ;;
    patchseq)
      rc=0
      : > "$log"
      for seed in $seeds; do "${fz[@]}" --seed "$seed" --crate patchseq >> "$log" 2>&1 || rc=1; done
      (exit $rc) ;;
    derive) python3 scripts/derive_sweep.py --count 100 > "$log" 2>&1 && python3 scripts/derive_sweep.py --count 100 --seed 1001 --resolver >> "$log" 2>&1 ;;
    tsedit) scripts/tsrust-full.sh --tsedit ${TSEDIT_ARGS:-} --out "$out/tsedit" > "$log" 2>&1 ;;
    *) echo "itempatch-sweep: unknown suite $s" >&2; exit 2 ;;
  esac
  rc=$?
  ok=$(grep -Ec '^(itempatch|resolve) oracle: ok' "$LUST_ITEMPATCH_LOG" 2>/dev/null || true)
  fail=$(grep -Evc '^(itempatch|resolve) oracle: ok' "$LUST_ITEMPATCH_LOG" 2>/dev/null || true)
  cold=0
  case $s in difftest|crates) cold=1 ;; esac
  [ "$rc" = 0 ] && [ "${fail:-0}" = 0 ] || bad=1
  if [ "$cold" = 0 ] && [ "${ok:-0}" = 0 ]; then
    echo "sweep: $s: the patch oracle logged 0 checks (a blind spot)"
    bad=1
  fi
  tail -3 "$log"
  grep -h '^sigops:\|^paths:' "$log" | head -6
  [ "${fail:-0}" = 0 ] || grep -Ev '^(itempatch|resolve) oracle: ok' "$LUST_ITEMPATCH_LOG" | sort | uniq -c | sort -rn | head -5
  # Coverage: oracle checks by kind and path (path=item is the item patch).
  [ "$cold" = 1 ] || echo "sweep: $s checks by path: $(grep -E '^(itempatch|resolve) oracle: ok' "$LUST_ITEMPATCH_LOG" 2>/dev/null | awk '{print $1"/"$4"/"$5}' | sort | uniq -c | sort -rn | tr -s ' ' | tr '\n' ';')"
  if [ "$cold" = 1 ]; then ok="n/a (cold suite)"; fi
  echo "sweep: $s rc=$rc oracle_ok=${ok:-0} oracle_fail=${fail:-0}"
done
exit $bad
