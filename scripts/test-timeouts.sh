#!/usr/bin/env bash
# Planted hang/slow proof for the one timeout rule (scripts/timeout-lib.sh,
# scripts/case-pool.sh): a lust compile that times out is retried once, alone,
# at a much longer limit; only a second timeout is a HANG and fails the
# script; a finished retry is reported as "slow" and is not a failure.
#
# How: target/release/lust is swapped for a wrapper (restored on exit) that,
# for calls selected by FAKE_LUST_* env vars, either sleeps forever (hang) or
# sleeps once past the case limit and then runs the real lust (slow). Each
# gate script then runs one small case with tiny timeout knobs, in both
# modes, and this script asserts the exit code, the HANG/slow line, and that
# the last line keeps the script's own summary form.
#
# Usage: scripts/test-timeouts.sh [filter]   (filter matches check names)
# Run it alone on a worktree or boat sandbox: it swaps the lust binary, so
# nothing else may use target/release/lust while it runs. It is not in the
# landing gate (gate-lib.sh has no step for it); the full run takes minutes,
# most of it test-client, test-driver, and smoke at their real limits.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" || exit 2
lust_bin=$root/target/release/lust
real=$root/target/release/lust.real-testtimeouts
if [ ! -x "$lust_bin" ]; then echo "error: $lust_bin not built. Run scripts/build.sh first."; exit 2; fi
if [ -e "$real" ]; then echo "error: $real exists (a killed earlier run?); restore $lust_bin by hand first"; exit 2; fi
filter=${1:-}
work=$(mktemp -d)
pass=0 fail=0 seen=0
echo "test-timeouts: work dir $work"

mv "$lust_bin" "$real"
restore() { rm -f "$lust_bin"; mv "$real" "$lust_bin"; }
trap restore EXIT
cat >"$lust_bin" <<EOF
#!/bin/sh
# Fake lust for scripts/test-timeouts.sh. See that script's header.
real="$real"
[ "\${1:-}" = daemon ] && exec "\$real" "\$@"
mode=\${FAKE_LUST_MODE:-off}
[ "\$mode" = off ] && exec "\$real" "\$@"
if [ -n "\${FAKE_LUST_ONLY:-}" ] && [ "\$1" != "\$FAKE_LUST_ONLY" ]; then exec "\$real" "\$@"; fi
if [ -n "\${FAKE_LUST_MATCH:-}" ]; then case " \$* " in *"\$FAKE_LUST_MATCH"*) ;; *) exec "\$real" "\$@" ;; esac; fi
if [ -n "\${FAKE_LUST_CWD:-}" ]; then case "\$PWD" in *"\$FAKE_LUST_CWD"*) ;; *) exec "\$real" "\$@" ;; esac; fi
if [ "\$mode" = hang ]; then exec sleep 3600; fi
state=\${FAKE_LUST_STATE:?slow mode needs FAKE_LUST_STATE}
[ -e "\$state" ] && exec "\$real" "\$@"
: >"\$state"
sleep "\${FAKE_LUST_SLEEP:-3}"
exec "\$real" "\$@"
EOF
chmod +x "$lust_bin"

# prove <name> <hang|slow> <last-line-regex> <cmd...>: run cmd planted in the
# given mode and assert the rule's three signs (exit code, HANG/slow line,
# the script's own last-line form). FAKE_LUST_* selectors and the script's
# timeout knobs ride in cmd's env (see the calls below).
prove() {
  local name=$1 mode=$2 want=$3; shift 3
  [[ -n "$filter" && "$name" != *"$filter"* ]] && return 0
  seen=$((seen + 1))
  rm -f "$work/state-$name"
  local out="$work/$name.$mode.log" rc ok=1 why=""
  FAKE_LUST_STATE="$work/state-$name" FAKE_LUST_MODE="$mode" timeout 1200 "$@" >"$out" 2>&1; rc=$?
  if [ "$mode" = hang ]; then
    [ "$rc" -ne 0 ] || { ok=0; why="exit 0, want a failure"; }
    grep -qi 'hang' "$out" || { ok=0; why="no HANG line"; }
  else
    [ "$rc" -eq 0 ] || { ok=0; why="exit $rc, want 0"; }
    grep -q 'slow:' "$out" || { ok=0; why="no slow: line"; }
  fi
  tail -1 "$out" | grep -qE "$want" || { ok=0; why="$why; last line: $(tail -1 "$out")"; }
  if [ "$ok" = 1 ]; then
    pass=$((pass + 1)); echo "ok   $name-$mode"
  else
    fail=$((fail + 1)); echo "FAIL $name-$mode: $why (log $out)"; tail -8 "$out"
  fi
}

last_failed() { echo "^$1: [0-9]+ passed, [1-9][0-9]* failed$"; }
last_passed() { echo "^$1: [0-9]+ passed, 0 failed$"; }

# case-pool runners: a pass case and a both-reject case (the brief's bug: a
# lust check timeout counted as the matching reject).
for m in hang slow; do
  prove difftest-pass $m "$(if [ $m = hang ]; then last_failed difftest; else last_passed difftest; fi)" \
    env FAKE_LUST_MATCH=alias_builtin_paths CASE_TIMEOUT=4 RETRY_CASE_TIMEOUT=12 FAKE_LUST_SLEEP=6 \
    scripts/difftest.sh alias_builtin_paths
  prove difftest-neg $m "$(if [ $m = hang ]; then last_failed difftest; else last_passed difftest; fi)" \
    env FAKE_LUST_MATCH=adt_outlives_construct CASE_TIMEOUT=4 RETRY_CASE_TIMEOUT=12 FAKE_LUST_SLEEP=6 \
    scripts/difftest.sh adt_outlives_construct
  prove runtest $m "$(if [ $m = hang ]; then last_failed runtest; else last_passed runtest; fi)" \
    env FAKE_LUST_MATCH=alias_builtin_paths CASE_TIMEOUT=4 RETRY_CASE_TIMEOUT=12 FAKE_LUST_SLEEP=6 \
    scripts/runtest.sh alias_builtin_paths
  prove parse-fuzz $m "$([ $m = hang ] && echo '^failing inputs kept in ' || echo '^parse-fuzz: 20 cases, 0 bad$')" \
    env FAKE_LUST_MATCH=case-00000 PARSE_FUZZ_TIMEOUT=1 FAKE_LUST_SLEEP=2 \
    scripts/parse-fuzz.sh 20 1
done

# Corpus runners. test-adapters uses a case lust cleanly rejects: slow must
# still judge it unsupported, hang must not become unsupported.
for m in hang slow; do
  prove test-traits $m "^test-traits lust: [0-9]+ matched, [0-9]+ unsupported, 0 wrong$" \
    env FAKE_LUST_MATCH=assoc_eq_binding CASE_TIMEOUT=4 RETRY_CASE_TIMEOUT=12 FAKE_LUST_SLEEP=6 \
    LUST=target/release/lust scripts/test-traits.sh assoc_eq_binding
  prove test-adapters $m "^test-adapters lust: [0-9]+ matched, [0-9]+ unsupported, 0 wrong$" \
    env FAKE_LUST_MATCH=scalars_repr CASE_TIMEOUT=4 RETRY_CASE_TIMEOUT=12 FAKE_LUST_SLEEP=6 \
    LUST=target/release/lust scripts/test-adapters.sh scalars_repr
  prove test-macros $m "^lust expansion tokens: [0-9]+ match, 0 differ, [0-9]+ unsupported$" \
    env FAKE_LUST_MATCH=sem_cfg_builtins CASE_TIMEOUT=4 RETRY_CASE_TIMEOUT=12 FAKE_LUST_SLEEP=6 \
    LUST=target/release/lust scripts/test-macros.sh sem_cfg_builtins
  prove test-kernels $m "^lust report: [0-9]+ ok, $([ $m = hang ] && echo '[1-9][0-9]*' || echo 0) wrong, [0-9]+ unsupported \(unsupported does not fail\)$" \
    env FAKE_LUST_MATCH=fileref_guard CASE_TIMEOUT=2 RETRY_CASE_TIMEOUT=8 FAKE_LUST_SLEEP=3 \
    LUST=target/release/lust scripts/test-kernels.sh fileref_guard
  prove test-crates $m "$(if [ $m = hang ]; then last_failed test-crates; else last_passed test-crates; fi)" \
    env FAKE_LUST_MATCH=buildscript CASE_TIMEOUT=2 RETRY_CASE_TIMEOUT=10 FAKE_LUST_SLEEP=3 \
    scripts/test-crates.sh buildscript
done

# Slice cold checks (parallel pool, solo rerun) and the daemon soak (serial).
for m in hang slow; do
  prove slice-check $m "$(if [ $m = hang ]; then last_failed slice-check; else last_passed slice-check; fi)" \
    env FAKE_LUST_ONLY=check FAKE_LUST_MATCH=slice_main SLICE_CHECK_TIMEOUT=2 SLICE_CHECK_RETRY_TIMEOUT=20 FAKE_LUST_SLEEP=3 \
    scripts/slice-check.sh bench/slices/goport-util-small
  prove daemon-soak $m "^daemon-soak: 60 requests,.* $([ $m = hang ] && echo '[1-9][0-9]*' || echo 0) failed$" \
    env FAKE_LUST_MATCH=p1.rs CASE_TIMEOUT=2 RETRY_CASE_TIMEOUT=8 FAKE_LUST_SLEEP=3 \
    scripts/daemon-soak.sh 60
done

# Daemon/client suites. Their own limits are the knobs; only the first
# case's calls are planted (by cwd), the rest of each suite runs for real.
for m in hang slow; do
  prove test-client $m "$(if [ $m = hang ]; then last_failed test-client; else last_passed test-client; fi)" \
    env FAKE_LUST_CWD=/autostart CASE_TIMEOUT=3 RETRY_CASE_TIMEOUT=10 FAKE_LUST_SLEEP=5 \
    scripts/test-client.sh
  prove test-driver $m "$(if [ $m = hang ]; then last_failed test-driver; else last_passed test-driver; fi)" \
    env FAKE_LUST_CWD=lust-driver-slots TEST_DRIVER_TIMEOUT=5 FAKE_LUST_SLEEP=8 \
    scripts/test-driver.sh
  prove fuzz-smoke $m "$([ $m = hang ] && echo 'AssertionError: HANG' || echo '^smoke: all passed$')" \
    env FAKE_LUST_MATCH=tab.rs FAKE_LUST_SLEEP=25 \
    python3 scripts/fuzz/smoke.py
  prove fuzz $m "$([ $m = hang ] && echo '^SEED-CRATE FAILURE \(daemon hang\)' || echo '^bad: \{\}$')" \
    env FAKE_LUST_ONLY=check FAKE_LUST_MATCH=crate.rs FAKE_LUST_SLEEP=3 LUST_FUZZ_OUT="$work/fuzz-out-$m" \
    scripts/fuzz/run.sh --edits 5 --seed 7 -- --timeout 2
done

# port-scan-sync already had the full rule; its unit suite fakes the
# timeout, the retry, and (now) the slow line without a planted binary.
[[ -z "$filter" || "port-scan" == *"$filter"* ]] && {
  seen=$((seen + 1))
  if FAKE_LUST_MODE=off python3 scripts/test-port-scan-sync.py >"$work/port-scan.log" 2>&1 \
    && grep -q 'ok a judged retry is reported as slow' "$work/port-scan.log" \
    && grep -q 'ok timed-out probe is retried once and can pass' "$work/port-scan.log" \
    && grep -q 'ok still timed out after one retry, no third try' "$work/port-scan.log"; then
    pass=$((pass + 1)); echo "ok   port-scan"
  else
    fail=$((fail + 1)); echo "FAIL port-scan: unit timeout checks (log $work/port-scan.log)"; tail -8 "$work/port-scan.log"
  fi
}

if [ "$seen" -eq 0 ]; then echo "FAIL: no checks matched filter '${filter}'"; fail=1; fi
echo "test-timeouts: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
