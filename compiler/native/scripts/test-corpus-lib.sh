#!/usr/bin/env bash
# Shared corpus runner logic for scripts/test-traits.sh and
# scripts/test-adapters.sh (ARCH 9.3 runner table). Source this
# file, call corpus_init, then corpus_pass_dir / corpus_fail_dir
# per fixture directory, then corpus_finish.
#
# Modes:
#   default: rustc is the oracle. Pass cases must build, run,
#            and match their sibling .stdout. Fail cases must
#            be rejected with the token recorded in the fixture
#            header (`// rustc-error: E####` or stable text).
#   LUST=path: also try lust on every case and compare
#            accept/reject and output. Gaps are reported as
#            UNSUPPORTED (lust rejects a rustc-pass case) or
#            WRONG (lust accepts a rustc-fail case, or its
#            output differs). Gaps do not fail the run unless
#            LUST_STRICT=1. A lust compile that times out is
#            retried once at RETRY_CASE_TIMEOUT (default 6x;
#            scripts/timeout-lib.sh): a finished retry is
#            reported as "slow" and judged normally, a second
#            timeout is a HANG and fails the run.
# Knobs: CASE_TIMEOUT (seconds per compile, default 30),
# RUN_TIMEOUT (seconds per run, default 10), SANITIZE=1 (with
# LUST: rebuild the generated C under ASan/UBSan, as difftest).
# LUST_USE_DAEMON=1: run the lust side through a private daemon on
# a socket in the work dir (the MGP joint gate needs the daemon
# path, ARCH 9.3). Default is cold, for the reason below.
# A fixture directory that is missing or holds no .rs files
# fails the run; corpora never skip in silence (ARCH 9.3).

corpus_init() {
  corpus_root=$(cd "$(dirname "${BASH_SOURCE[1]}")/.." && pwd)
  corpus_work=$(mktemp -d)
  trap 'rm -rf "$corpus_work"' EXIT
  corpus_filter=${1:-}
  corpus_pass=0
  corpus_fail=0
  corpus_cases=0
  lust_match=0
  lust_unsupported=0
  lust_wrong=0
  lust_hang=0
  case_timeout=${CASE_TIMEOUT:-30}
  run_timeout=${RUN_TIMEOUT:-10}
  # The one timeout rule (lust_cc, is_timeout, $retry_timeout). This runner is
  # serial, so lust_cc's retry is alone.
  # shellcheck source=scripts/timeout-lib.sh
  source "${BASH_SOURCE[0]%/*}/timeout-lib.sh"
  # Compile cold, as difftest does: a warm daemon would make
  # accept/reject depend on session state, not on the corpus.
  # LUST_USE_DAEMON=1 opts into a private daemon instead: the
  # socket lives in the work dir, the idle exit is short, and a
  # two-check warm-up below makes every case daemon-served (the
  # client's first command is answered cold while it autostarts
  # the daemon; see src/client.rs).
  export LUST_DAEMON_SOCKET="$corpus_work/daemon.sock"
  export LUST_CACHE_DIR="$corpus_work/cache"
  if [ "${LUST_USE_DAEMON:-0}" = 1 ]; then
    unset LUST_NO_DAEMON
    export LUST_DAEMON_IDLE=${LUST_DAEMON_IDLE:-20s}
  else
    export LUST_NO_DAEMON=1
  fi
  if ! command -v timeout >/dev/null 2>&1; then
    echo "FAIL: the timeout command is required for per-case timeouts"
    exit 1
  fi
  if ! command -v rustc >/dev/null 2>&1; then
    echo "FAIL: rustc is required as the corpus oracle"
    exit 1
  fi
  if [ -n "${LUST:-}" ]; then
    case "$LUST" in
      /*) ;;
      *) LUST=$PWD/$LUST ;;
    esac
    if [ ! -x "$LUST" ]; then
      echo "FAIL: LUST=$LUST is not an executable file"
      exit 1
    fi
    echo "lust mode: comparing $LUST against rustc (gaps do not fail${LUST_STRICT:+ unless LUST_STRICT=1})"
    if [ "${LUST_USE_DAEMON:-0}" = 1 ]; then
      # Warm-up: check one probe file twice. The first check
      # autostarts the daemon, the second is served by it, so
      # every real case below takes the daemon path.
      echo 'fn main() {}' >"$corpus_work/warmup.rs"
      lust_cc "$LUST" check "$corpus_work/warmup.rs" >/dev/null 2>&1 || true
      if ! lust_cc "$LUST" check "$corpus_work/warmup.rs" >/dev/null 2>"$corpus_work/warmup.err"; then
        echo "FAIL: daemon warm-up check failed: $(head -3 "$corpus_work/warmup.err")"
        exit 1
      fi
      echo "lust daemon: private daemon on $LUST_DAEMON_SOCKET (warm-up done)"
    fi
  fi
}

corpus_report() {
  if [ "$1" = ok ]; then
    corpus_pass=$((corpus_pass + 1))
  else
    corpus_fail=$((corpus_fail + 1))
    echo "FAIL $2: $3"
  fi
}

corpus_lust_note() {
  # $1 = match|unsupported|wrong|hang, $2 = case, $3 = detail. A hang is not a
  # gap: corpus_finish adds every hang to the failed count.
  case "$1" in
    match) lust_match=$((lust_match + 1)) ;;
    unsupported)
      lust_unsupported=$((lust_unsupported + 1))
      echo "UNSUPPORTED $2: $3"
      ;;
    wrong)
      lust_wrong=$((lust_wrong + 1))
      echo "WRONG $2: $3"
      ;;
    hang)
      lust_hang=$((lust_hang + 1))
      echo "HANG $2: $3"
      ;;
  esac
}

# corpus_slow_note <case> <step>: after a lust_cc that needed its retry and
# finished, say so. The case is then judged on the retry's result as usual.
corpus_slow_note() {
  [ "$lust_cc_slow" = 1 ] && echo "slow: $1 $2 timed out at ${case_timeout}s, finished alone at ${retry_timeout}s"
  return 0
}

corpus_require_dir() {
  # $1 = dir relative to the repo root
  local dir=$corpus_root/$1
  if [ ! -d "$dir" ]; then
    corpus_report bad "$1" "fixture directory is missing"
    return 1
  fi
  if ! compgen -G "$dir/*.rs" >/dev/null; then
    corpus_report bad "$1" "fixture directory holds no .rs files"
    return 1
  fi
  return 0
}

corpus_rustc_flags=(--edition 2024 -C overflow-checks=on -A warnings)

corpus_pass_dir() {
  # $1 = dir relative to the repo root holding runnable cases
  corpus_require_dir "$1" || return 0
  local dir=$corpus_root/$1 f name
  for f in "$dir"/*.rs; do
    name=$(basename "$f" .rs)
    [[ -n "$corpus_filter" && "$name" != *"$corpus_filter"* ]] && continue
    corpus_cases=$((corpus_cases + 1))
    corpus_pass_one "$f" "$name"
  done
}

corpus_pass_one() {
  local f=$1 name=$2
  local expected=${f%.rs}.stdout
  if [ ! -f "$expected" ]; then
    corpus_report bad "$name" "missing expected stdout file ${expected##*/}"
    return
  fi
  timeout "$case_timeout" rustc "${corpus_rustc_flags[@]}" -o "$corpus_work/$name.rustc" "$f" 2>"$corpus_work/$name.rustc.err"
  local rustc_status=$?
  if [ $rustc_status -eq 124 ]; then
    corpus_report bad "$name" "rustc timed out after ${case_timeout}s on a pass case"
    return
  fi
  if [ $rustc_status -ne 0 ]; then
    corpus_report bad "$name" "rustc rejected a pass case: $(head -3 "$corpus_work/$name.rustc.err" | tr '\n' ' ')"
    return
  fi
  timeout "$run_timeout" "$corpus_work/$name.rustc" >"$corpus_work/$name.out" 2>"$corpus_work/$name.run.err"
  local status=$?
  if [ $status -ne 0 ]; then
    corpus_report bad "$name" "rustc build exited with status $status"
    return
  fi
  if ! cmp -s "$expected" "$corpus_work/$name.out"; then
    corpus_report bad "$name" "stdout differs from ${expected##*/}:"
    diff "$expected" "$corpus_work/$name.out" | head -10
    return
  fi
  corpus_report ok "$name"
  [ -n "${LUST:-}" ] || return
  # Lust path: build, run, and compare to the same oracle output.
  lust_cc "$LUST" build "$f" -o "$corpus_work/$name.lust" 2>"$corpus_work/$name.lust.err"
  status=$?
  if is_timeout "$status"; then
    corpus_lust_note hang "$name" "lust build timed out alone at ${retry_timeout}s (first at ${case_timeout}s)"
    return
  fi
  corpus_slow_note "$name" "lust build"
  if [ "$status" -ne 0 ]; then
    corpus_lust_note unsupported "$name" "lust rejected a rustc-pass case: $(head -2 "$corpus_work/$name.lust.err" | tr '\n' ' ')"
    return
  fi
  timeout "$run_timeout" "$corpus_work/$name.lust" >"$corpus_work/$name.lust.out" 2>/dev/null
  status=$?
  if [ $status -ne 0 ] || ! cmp -s "$expected" "$corpus_work/$name.lust.out"; then
    corpus_lust_note wrong "$name" "lust output differs from rustc (exit $status)"
    return
  fi
  if [ "${SANITIZE:-0}" = 1 ]; then
    lust_cc "$LUST" build "$f" -o "$corpus_work/$name.tmp" --emit-c "$corpus_work/$name.san" 2>/dev/null
    status=$?
    if is_timeout "$status"; then
      corpus_lust_note hang "$name" "lust build --emit-c timed out alone at ${retry_timeout}s (first at ${case_timeout}s)"
      return
    fi
    corpus_slow_note "$name" "lust build --emit-c"
    if [ "$status" -ne 0 ]; then
      corpus_lust_note wrong "$name" "lust build --emit-c failed on a case the plain build passed"
      return
    fi
    if ! gcc -O1 -w -fno-sanitize-recover=undefined -fsanitize=address,undefined -o "$corpus_work/$name.sanbin" "$corpus_work/$name.san".*.c 2>"$corpus_work/$name.san.err"; then
      corpus_lust_note wrong "$name" "gcc could not build the generated C"
      return
    fi
    timeout "$run_timeout" "$corpus_work/$name.sanbin" >"$corpus_work/$name.san.out" 2>"$corpus_work/$name.san.run.err"
    if grep -q "runtime error\|Sanitizer" "$corpus_work/$name.san.run.err" || ! cmp -s "$expected" "$corpus_work/$name.san.out"; then
      corpus_lust_note wrong "$name" "sanitized build failed or output differs"
      return
    fi
  fi
  corpus_lust_note match "$name" ""
}

corpus_fail_dir() {
  # $1 = dir relative to the repo root holding rejected cases
  corpus_require_dir "$1" || return 0
  local dir=$corpus_root/$1 f name
  for f in "$dir"/*.rs; do
    name=$(basename "$f" .rs)
    [[ -n "$corpus_filter" && "$name" != *"$corpus_filter"* ]] && continue
    corpus_cases=$((corpus_cases + 1))
    corpus_fail_one "$f" "$name"
  done
}

corpus_fail_one() {
  local f=$1 name=$2
  local want
  want=$(grep -m1 'rustc-error:' "$f" | sed 's|.*rustc-error: *||; s/[[:space:]]*$//')
  if [ -z "$want" ]; then
    corpus_report bad "$name" "no // rustc-error: header records the rustc error class"
    return
  fi
  timeout "$case_timeout" rustc "${corpus_rustc_flags[@]}" -o "$corpus_work/$name.rustc" "$f" 2>"$corpus_work/$name.rustc.err"
  local status=$?
  if [ $status -eq 0 ]; then
    corpus_report bad "$name" "rustc accepted a fail case (recorded class: $want)"
    return
  fi
  if [ $status -eq 124 ]; then
    corpus_report bad "$name" "rustc timed out after ${case_timeout}s instead of rejecting"
    return
  fi
  if ! grep -qF "$want" "$corpus_work/$name.rustc.err"; then
    corpus_report bad "$name" "rustc rejected but without recorded class '$want': $(grep -o 'E[0-9]\{4\}' "$corpus_work/$name.rustc.err" | sort -u | tr '\n' ' ')"
    return
  fi
  corpus_report ok "$name"
  [ -n "${LUST:-}" ] || return
  lust_cc "$LUST" check --json "$f" >"$corpus_work/$name.lust.out" 2>&1
  status=$?
  if is_timeout "$status"; then
    # A timeout is never a reject, not even when rustc rejects the case too.
    corpus_lust_note hang "$name" "lust check timed out alone at ${retry_timeout}s (first at ${case_timeout}s)"
    return
  fi
  corpus_slow_note "$name" "lust check"
  if [ $status -eq 0 ]; then
    # A post-monomorphization error (a generic inline const assert,
    # ARCH 2.13) is a build error only, in rustc's check as in lust's:
    # the build must reject it then.
    lust_cc "$LUST" build "$f" -o "$corpus_work/$name.lust" >"$corpus_work/$name.lust.out" 2>&1
    status=$?
    if is_timeout "$status"; then
      corpus_lust_note hang "$name" "lust build timed out alone at ${retry_timeout}s (first at ${case_timeout}s)"
    elif [ $status -eq 0 ]; then
      corpus_slow_note "$name" "lust build"
      corpus_lust_note wrong "$name" "lust accepted a case rustc rejects ($want)"
    else
      corpus_slow_note "$name" "lust build"
      corpus_lust_note match "$name" ""
    fi
  else
    corpus_lust_note match "$name" ""
  fi
}

corpus_finish() {
  # $1 = runner name for the summary line
  if [ "$corpus_cases" -eq 0 ] && [ "$corpus_fail" -eq 0 ]; then
    echo "FAIL: no cases matched filter '${corpus_filter}'"
    corpus_fail=1
  fi
  # A hang fails the run (it is not an unsupported gap). The summary lines keep
  # their form: the hang count joins the failed count of the first line, and
  # the HANG lines above name the cases.
  corpus_fail=$((corpus_fail + lust_hang))
  echo "$1: $corpus_pass passed, $corpus_fail failed (rustc oracle)"
  if [ -n "${LUST:-}" ]; then
    echo "$1 lust: $lust_match matched, $lust_unsupported unsupported, $lust_wrong wrong"
    if [ "${LUST_STRICT:-0}" = 1 ] && [ $((lust_unsupported + lust_wrong)) -ne 0 ]; then
      [ "$corpus_fail" -eq 0 ] && corpus_fail=1
    fi
  fi
  [ "$corpus_fail" -eq 0 ]
}
