#!/usr/bin/env bash
# The macro corpus runner (ARCH 6.5, 9.3). Three layers per fixture:
#   (a) token compare: normalized tokens of `rustc -Zunpretty=expanded`
#       stored in the sibling .expanded file (tests/macros/normalize.py).
#   (b) executable cases: the rustc build must run and print the sibling
#       .out file. A `// mode: test` fixture is also built with --test
#       and its tests must pass.
#   (c) both-reject cases in tests/macros/reject: rustc must reject and
#       its stderr must contain the `// rustc-error:` class recorded in
#       the fixture header.
# ts-rust coverage: every name in tests/macros/ts-rust-names.txt must
# appear in a pass fixture `// macro:` header.
#
# Usage: scripts/test-macros.sh [filter]
#   UPDATE=1  regenerate .out and .expanded from the current rustc.
#   TIMEOUT   per-case seconds for each rustc, lust and run step (15).
#   LUST=path also build and run pass cases with lust and check reject
#             cases with `lust check`. Lust rejecting a user macro is
#             reported as "unsupported" and does not fail the run. A lust
#             compile that times out is retried once at RETRY_CASE_TIMEOUT
#             (default 6x; scripts/timeout-lib.sh): a finished retry is
#             reported as "slow" and judged normally, a second timeout is
#             a HANG and fails the run.
#             A lust build whose output differs, or that accepts a
#             both-reject case, is counted as WRONG (it breaks rule
#             zero). WRONG counts do not fail the run until the
#             W-MACRO gate: set LUST_STRICT=1 to make them fail, as
#             in scripts/test-corpus-lib.sh. If the binary has an
#             `expand` command, its token stream is compared with
#             .expanded and the match and differ counts are reported.
#             LUST_STRICT=1 also makes a token mismatch fail.
#   SANITIZE=1 (with LUST) also rebuilds each supported pass case's
#             generated C under ASan/UBSan and reruns it, as difftest
#             does (ARCH 9.3: every runner works with SANITIZE=1 where
#             it builds C).
set -uo pipefail
# Resolve a relative LUST path against the caller's directory
# before changing to the repo root, as test-corpus-lib.sh does.
if [ -n "${LUST:-}" ]; then
  case "$LUST" in
    /*) ;;
    *) LUST=$PWD/$LUST ;;
  esac
fi
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root" # fixture paths stay relative so file! and panic paths are stable

for d in tests/macros/pass tests/macros/reject; do
  if [ ! -d "$d" ]; then
    echo "FAIL fixture dir missing: $d"
    exit 1
  fi
done
for f in tests/macros/normalize.py tests/macros/ts-rust-names.txt; do
  if [ ! -f "$f" ]; then
    echo "FAIL corpus file missing: $f"
    exit 1
  fi
done

for d in tests/macros/pass tests/macros/reject; do
  if [ -z "$(find "$d" -name '*.rs' -print -quit)" ]; then
    echo "FAIL fixture dir holds no .rs files: $d"
    exit 1
  fi
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# Fixed compile-time env: sem_cfg_builtins prints env! of this name.
export LUST_MACRO_TEST_VALUE=hello42
# Per-case timeouts are an ARCH 9.3 runner requirement.
if ! command -v timeout >/dev/null 2>&1; then
  echo "FAIL: the timeout command is required for per-case timeouts"
  exit 1
fi
# CASE_TIMEOUT per compile and RUN_TIMEOUT per run, as in
# test-corpus-lib.sh. TIMEOUT sets both.
case_timeout=${CASE_TIMEOUT:-${TIMEOUT:-15}}
run_timeout=${RUN_TIMEOUT:-${TIMEOUT:-15}}
# The one timeout rule for lust compiles (lust_cc). This runner is serial, so
# the retry is alone. Rustc steps keep plain run_cc.
# shellcheck source=scripts/timeout-lib.sh
source "$root/scripts/timeout-lib.sh"
run_cc() { timeout "$case_timeout" "$@"; }
run_to() { timeout "$run_timeout" "$@"; }
rustc_flags=(--edition 2024 -C overflow-checks=on -A warnings)
filter=${1:-}
pass=0
fail=0
lust_supported=0
lust_wrong=0
lust_unsupported=0
lust_hang=0
lust_rejected=0
lust_expand_match=0
lust_expand_differ=0
lust_expand_unsupported=0

report() {
  if [ "$1" = ok ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL $2: $3"; fi
}

# hang_note <case> <step>: a lust compile timed out twice. Counted apart here
# and added to $fail at the end, so the summary lines keep their form.
hang_note() { lust_hang=$((lust_hang + 1)); echo "HANG $1: $2 timed out alone at ${retry_timeout}s (first at ${case_timeout}s)"; }
# slow_note <case> <step>: a lust_cc needed its retry and finished.
slow_note() { [ "$lust_cc_slow" = 1 ] && echo "slow: $1 $2 timed out at ${case_timeout}s, finished alone at ${retry_timeout}s"; return 0; }

# Probe once whether the lust binary offers an expand command.
lust_expand_ok=0
if [ -n "${LUST:-}" ]; then
  if [ ! -x "$LUST" ]; then
    echo "FAIL LUST=$LUST is not executable"
    exit 1
  fi
  export LUST_NO_DAEMON=1
  export LUST_DAEMON_SOCKET="$work/no-daemon.sock"
  export LUST_CACHE_DIR="$work/cache"
  echo 'fn main() {}' >"$work/probe.rs"
  lust_cc "$LUST" expand "$work/probe.rs" >"$work/probe.exp" 2>/dev/null
  probe_st=$?
  if is_timeout "$probe_st"; then
    echo "FAIL probe: lust expand timed out alone at ${retry_timeout}s (hang)"
    exit 1
  fi
  slow_note probe "lust expand"
  if [ "$probe_st" -eq 0 ] && [ -s "$work/probe.exp" ]; then
    lust_expand_ok=1
  fi
fi

lust_pass_case() {
  # $1 fixture, $2 expected .out, $3 expected .expanded, $4 name
  local f=$1 out=$2 expanded=$3 name=$4
  # The .out files come from rustc's default build, which unwinds: a
  # case that catches a panic (catch_unwind) needs lust's unwind mode.
  lust_cc "$LUST" build -C panic=unwind "$f" -o "$work/$name.lust" 2>"$work/$name.lust.err"
  local st=$?
  if is_timeout "$st"; then
    hang_note "$name" "lust build"
    return 1
  fi
  slow_note "$name" "lust build"
  if [ "$st" -ne 0 ]; then
    lust_unsupported=$((lust_unsupported + 1))
    return 0
  fi
  run_to "$work/$name.lust" >"$work/$name.lust.out" 2>"$work/$name.lust.runerr"
  local st=$?
  if [ $st -ne 0 ]; then
    echo "WRONG $name: lust build exits $st: $(head -3 "$work/$name.lust.runerr")"
    lust_wrong=$((lust_wrong + 1))
    if [ "${LUST_STRICT:-0}" = 1 ]; then report bad "$name" "lust wrong result in strict mode"; return 1; fi
    return 0
  fi
  if ! diff -q "$out" "$work/$name.lust.out" >/dev/null; then
    echo "WRONG $name: lust output differs from rustc:"
    diff "$out" "$work/$name.lust.out" | head -10
    lust_wrong=$((lust_wrong + 1))
    if [ "${LUST_STRICT:-0}" = 1 ]; then report bad "$name" "lust wrong result in strict mode"; return 1; fi
    return 0
  fi
  lust_supported=$((lust_supported + 1))
  if [ "${SANITIZE:-0}" = 1 ]; then
    # The daemon is off in this runner, so --emit-c writes every unit.
    lust_cc "$LUST" build -C panic=unwind "$f" -o "$work/$name.tmp" --emit-c "$work/$name.san" 2>/dev/null
    st=$?
    if is_timeout "$st"; then
      hang_note "$name" "lust build --emit-c"
      return 1
    fi
    slow_note "$name" "lust build --emit-c"
    if [ "$st" -ne 0 ]; then
      echo "WRONG $name: lust build --emit-c failed under SANITIZE=1 on a case the plain build passed"
      lust_wrong=$((lust_wrong + 1))
      if [ "${LUST_STRICT:-0}" = 1 ]; then report bad "$name" "lust wrong result in strict mode"; return 1; fi
      return 0
    fi
    if ! gcc -O1 -w -fno-sanitize-recover=undefined -fsanitize=address,undefined -o "$work/$name.sanbin" "$work/$name.san".*.c 2>"$work/$name.san.err"; then
      echo "WRONG $name: gcc could not build the generated C under SANITIZE=1: $(head -3 "$work/$name.san.err")"
      lust_wrong=$((lust_wrong + 1))
      if [ "${LUST_STRICT:-0}" = 1 ]; then report bad "$name" "lust wrong result in strict mode"; return 1; fi
      return 0
    fi
    run_to "$work/$name.sanbin" >"$work/$name.san.out" 2>"$work/$name.san.runerr"
    if grep -q "runtime error\|Sanitizer" "$work/$name.san.runerr" || ! diff -q "$out" "$work/$name.san.out" >/dev/null; then
      echo "WRONG $name: sanitized build failed or output differs: $(head -3 "$work/$name.san.runerr")"
      lust_wrong=$((lust_wrong + 1))
      if [ "${LUST_STRICT:-0}" = 1 ]; then report bad "$name" "lust wrong result in strict mode"; return 1; fi
      return 0
    fi
  fi
  if [ "$lust_expand_ok" = 1 ]; then
    # rustc prints derive expansions as generated impls, which lust has
    # no token form for (normalize.py): such a fixture can only differ,
    # so its token compare counts as unsupported.
    if grep -q 'automatically_derived' "$expanded"; then
      lust_expand_unsupported=$((lust_expand_unsupported + 1))
    elif lust_cc "$LUST" expand "$f" >"$work/$name.raw.exp" 2>/dev/null; st=$?; is_timeout "$st"; then
      hang_note "$name" "lust expand"
      return 1
    elif [ "$st" -eq 0 ] && python3 tests/macros/normalize.py <"$work/$name.raw.exp" >"$work/$name.lust.expanded" \
      && [ -s "$work/$name.lust.expanded" ]; then
      slow_note "$name" "lust expand"
      if diff -q "$expanded" "$work/$name.lust.expanded" >/dev/null; then
        lust_expand_match=$((lust_expand_match + 1))
      elif [ "${LUST_STRICT:-0}" = 1 ]; then
        report bad "$name" "lust expansion tokens differ from rustc .expanded"
        return 1
      else
        lust_expand_differ=$((lust_expand_differ + 1))
      fi
    else
      lust_expand_unsupported=$((lust_expand_unsupported + 1))
    fi
  fi
  return 0
}

# ---- layer (a) and (b): executable fixtures ----
while IFS= read -r f; do
  name=${f#tests/macros/pass/}
  name=${name%.rs}
  flat=$(echo "$name" | tr '/' '_')
  [[ -n "$filter" && "$name" != *"$filter"* ]] && continue
  case_fail=0
  if ! run_cc rustc "${rustc_flags[@]}" -o "$work/$flat.rustc" "$f" 2>"$work/$flat.rustc.err"; then
    report bad "$name" "rustc rejected it: $(grep -m3 '^error' "$work/$flat.rustc.err")"
    continue
  fi
  run_to "$work/$flat.rustc" >"$work/$flat.rustc.out" 2>"$work/$flat.rustc.runerr"
  st=$?
  if [ $st -ne 0 ]; then
    report bad "$name" "rustc build exits $st: $(head -3 "$work/$flat.rustc.runerr")"
    continue
  fi
  if [ "${UPDATE:-0}" = 1 ]; then
    cp "$work/$flat.rustc.out" "${f%.rs}.out"
  elif [ ! -f "${f%.rs}.out" ]; then
    report bad "$name" "missing ${f%.rs}.out (run with UPDATE=1)"
    case_fail=1
  elif ! diff -q "${f%.rs}.out" "$work/$flat.rustc.out" >/dev/null; then
    report bad "$name" "run output differs from .out:"
    diff "${f%.rs}.out" "$work/$flat.rustc.out" | head -10
    case_fail=1
  fi
  if grep -q '^// mode: test' "$f"; then
    if ! run_cc rustc "${rustc_flags[@]}" --test -o "$work/$flat.test" "$f" 2>"$work/$flat.test.err"; then
      report bad "$name" "rustc --test rejected it: $(grep -m3 '^error' "$work/$flat.test.err")"
      case_fail=1
    elif ! run_to "$work/$flat.test" >"$work/$flat.test.out" 2>&1 || ! grep -q 'test result: ok' "$work/$flat.test.out"; then
      report bad "$name" "--test run failed: $(tail -3 "$work/$flat.test.out")"
      case_fail=1
    fi
  fi
  if RUSTC_BOOTSTRAP=1 run_cc rustc "${rustc_flags[@]}" -Zunpretty=expanded "$f" >"$work/$flat.raw.exp" 2>"$work/$flat.exp.err"; then
    python3 tests/macros/normalize.py <"$work/$flat.raw.exp" >"$work/$flat.expanded"
    if [ "${UPDATE:-0}" = 1 ]; then
      cp "$work/$flat.expanded" "${f%.rs}.expanded"
    elif [ ! -f "${f%.rs}.expanded" ]; then
      report bad "$name" "missing ${f%.rs}.expanded (run with UPDATE=1)"
      case_fail=1
    elif ! diff -q "${f%.rs}.expanded" "$work/$flat.expanded" >/dev/null; then
      report bad "$name" "expansion tokens differ from .expanded (first diff):"
      diff "${f%.rs}.expanded" "$work/$flat.expanded" | head -6
      case_fail=1
    fi
  else
    report bad "$name" "rustc -Zunpretty=expanded failed: $(grep -m3 '^error' "$work/$flat.exp.err")"
    case_fail=1
  fi
  if [ -n "${LUST:-}" ]; then
    lust_pass_case "$f" "${f%.rs}.out" "${f%.rs}.expanded" "$flat" || case_fail=1
  fi
  [ "$case_fail" = 0 ] && report ok "$name"
done < <(find tests/macros/pass -name '*.rs' | sort)

# ---- layer (c): both-reject fixtures ----
for f in tests/macros/reject/*.rs; do
  name=$(basename "$f" .rs)
  [[ -n "$filter" && "$name" != *"$filter"* ]] && continue
  errclass=$(sed -n 's|^// rustc-error: ||p' "$f" | head -1)
  if [ -z "$errclass" ]; then
    report bad "$name" "no // rustc-error: class recorded in the header"
    continue
  fi
  if run_cc rustc "${rustc_flags[@]}" -o "$work/$name.rustc" "$f" 2>"$work/$name.rustc.err"; then
    report bad "$name" "rustc accepted it, so it is not a both-reject case"
    continue
  fi
  if ! grep -qF "$errclass" "$work/$name.rustc.err"; then
    report bad "$name" "rustc error class changed, want '$errclass', got: $(grep -m2 '^error' "$work/$name.rustc.err")"
    continue
  fi
  if [ -n "${LUST:-}" ]; then
    lust_cc "$LUST" check "$f" >"$work/$name.lust.out" 2>&1
    st=$?
    if is_timeout "$st"; then
      # A timeout is never a reject, not even when rustc rejects the case too.
      hang_note "$name" "lust check"
      continue
    fi
    slow_note "$name" "lust check"
    if [ "$st" -eq 0 ]; then
      echo "WRONG $name: lust accepted a program rustc rejects"
      lust_wrong=$((lust_wrong + 1))
      if [ "${LUST_STRICT:-0}" = 1 ]; then
        report bad "$name" "lust wrong result in strict mode"
      else
        report ok "$name"
      fi
      continue
    fi
    lust_rejected=$((lust_rejected + 1))
  fi
  report ok "$name"
done

# ---- ts-rust macro name coverage (skipped under a filter) ----
if [ -z "$filter" ]; then
  covered=$(grep -h '^// macro:' $(find tests/macros/pass -name '*.rs' | sort) | sed 's|^// macro: ||' | tr ',' '\n' | sed 's/^ *//; s/ *$//' | sort -u)
  missing=0
  total=0
  while IFS= read -r want; do
    case "$want" in '' | \#*) continue ;; esac
    total=$((total + 1))
    if ! grep -qx "$want" <<<"$covered"; then
      echo "FAIL coverage: ts-rust macro '$want' has no fixture"
      missing=$((missing + 1))
    fi
  done <tests/macros/ts-rust-names.txt
  echo "coverage: $((total - missing))/$total ts-rust macro names have fixtures"
  [ "$missing" -ne 0 ] && fail=$((fail + missing))
fi

# A hang fails the run. The summary lines keep their form: the hang count
# joins the failed count, and the HANG lines above name the cases.
fail=$((fail + lust_hang))
echo "test-macros: $pass passed, $fail failed"
if [ -n "${LUST:-}" ]; then
  echo "lust: $lust_supported supported, $lust_unsupported unsupported, $lust_wrong wrong, $lust_rejected both-reject cases rejected"
  if [ "$lust_expand_ok" = 1 ]; then
    echo "lust expansion tokens: $lust_expand_match match, $lust_expand_differ differ, $lust_expand_unsupported unsupported"
  else
    echo "lust expansion tokens: no lust expand command, compare skipped"
  fi
fi
[ "$fail" -eq 0 ]
