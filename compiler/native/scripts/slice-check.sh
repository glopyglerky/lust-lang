#!/usr/bin/env bash
# slice-check.sh [slice-dir]: the pinned-slice regression test (fx-stdos).
#
# With no argument (the gate form):
#   1. Cold `lust check` (no daemon, fresh cache, check only) of EVERY
#      pinned slice in bench/slices/ (every dir with prune.json and src/),
#      in parallel, each with the dev features its prune.json records
#      (`dev_features`, replayed as --features like speed_slice.py does).
#   2. A full build and run of the smallest slice only, with the bench
#      driver mounted and stdout compared to a rustc build, as in round 1.
# With a slice dir argument: only that slice, old form (cold check plus
# the full build and run), for direct debugging.
#
# Known broken slices are listed in scripts/slice-check.expected-fail
# (one per line: "<slice-name> <reason>"; blank lines and #-comments
# are skipped). An expected-fail slice that fails its cold check is
# reported but not counted. An expected-fail slice that PASSES is a
# failure: remove it from the list, so the list must shrink as fixes
# land. A name in the list that is not a pinned slice is also a failure.
#
# Env knobs: SLICE_CHECK_JOBS (parallel cold checks, default 4),
# SLICE_CHECK_TIMEOUT (seconds per cold check, default 120),
# SLICE_CHECK_RETRY_TIMEOUT (seconds for the solo rerun of a timed-out cold
# check, default 6x; a finished rerun is reported as "slow" and judged
# normally, a second timeout is a HANG failure even for an expected-fail
# slice: a timeout is never a reject),
# SLICE_CHECK_EXPECTED_FAIL (path override, for tests).
# The last line is the gate summary: "slice-check: N passed, M failed".
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
lust=$root/target/release/lust
expected_file=${SLICE_CHECK_EXPECTED_FAIL:-$root/scripts/slice-check.expected-fail}
jobs=${SLICE_CHECK_JOBS:-4}
timeout_secs=${SLICE_CHECK_TIMEOUT:-120}
retry_secs=${SLICE_CHECK_RETRY_TIMEOUT:-$((timeout_secs * 6))}
# is_timeout_rc <status>: a timeout (124) or a kill after one (137), as in
# scripts/timeout-lib.sh.
is_timeout_rc() { [ "$1" = 124 ] || [ "$1" = 137 ]; }
passed=0
failed=0
wall_start=$(date +%s%N)

summary() { echo "slice-check: $passed passed, $failed failed"; }
if [ ! -x "$lust" ]; then
  echo "error: $lust not built. Run scripts/build.sh first."
  failed=1
  summary
  exit 1
fi

work=$(mktemp -d "${TMPDIR:-/tmp}/lust-slice-check-XXXXXX")
trap 'rm -rf "$work"' EXIT
export LUST_NO_DAEMON=1

# slice_meta <slice-dir>: prints "<root-file> <comma-features-or--> <lines>"
# from prune.json (spec.root, dev_features, lines).
slice_meta() {
  python3 -c '
import json, sys
d = json.load(open(sys.argv[1]))
feats = ",".join(d.get("dev_features") or []) or "-"
print(d["spec"]["root"], feats, d.get("lines") or 0)
' "$1/prune.json" 2>/dev/null
}

# cold_check <name> <slice-dir> <root-file> <features>: copies the slice
# to its own work dir and runs a cold lust check there. Writes
# $work/check-<name>.log and .rc (124 on timeout). Runs in parallel.
cold_check() {
  local name=$1 dir=$2 root_name=$3 feats=$4
  local w=$work/check-$name
  mkdir -p "$w"
  cp -r "$dir/src" "$w/src"
  local args=()
  [ "$feats" = "-" ] || args+=(--features "$feats")
  local start end
  start=$(date +%s%N)
  LUST_CACHE_DIR="$w/cold-cache" timeout -k 5 "$timeout_secs" \
    "$lust" check "$w/src/$root_name" "${args[@]}" >"$w.log" 2>&1
  echo $? >"$w.rc"
  end=$(date +%s%N)
  echo $(( (end - start) / 1000000 )) >"$w.ms"
}

# cold_retry <name> <slice-dir> <root-file> <features>: rerun one timed-out
# cold check alone at $retry_secs (the caller runs it serially, after the
# parallel pool). Overwrites check-<name>.rc and .ms. Leaves
# check-<name>.hang when the rerun timed out too, else prints a slow line.
cold_retry() {
  local name=$1
  rm -rf "$work/check-$name" "$work/check-$name.hang"
  (timeout_secs=$retry_secs cold_check "$@")
  if is_timeout_rc "$(cat "$work/check-$name.rc")"; then
    : >"$work/check-$name.hang"
  else
    echo "slow: check $name timed out at ${timeout_secs}s under load, finished alone at ${retry_secs}s"
  fi
}

# run_step <name> <command...>: counts one test, prints its time.
run_step() {
  local name=$1
  shift
  local start end
  start=$(date +%s%N)
  if "$@" >"$work/$name.log" 2>&1; then
    end=$(date +%s%N)
    echo "$name: ok ($(( (end - start) / 1000000 )) ms)"
    passed=$((passed + 1))
    return 0
  fi
  end=$(date +%s%N)
  echo "$name: FAILED ($(( (end - start) / 1000000 )) ms); first lines of the log:"
  head -5 "$work/$name.log"
  failed=$((failed + 1))
  return 1
}

# build_and_run <name> <slice-dir> <root-file> <features>: the round 1
# full test (bench driver mounted, rustc oracle, lust build, stdout
# compare). Counts two tests: rustc-oracle and lust-build-run.
build_and_run() {
  local name=$1 dir=$2 root_name=$3 feats=$4
  local w=$work/build-$name
  mkdir -p "$w"
  cp -r "$dir/src" "$w/src"
  local entry=$w/src/$root_name
  if ! python3 - "$entry" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
text = p.read_text()
if text.count("fn main() {}") != 1:
    sys.exit(1)
p.write_text(text.replace("fn main() {}", "mod bench_driver;\n\nfn main() {\n    bench_driver::run();\n}"))
PY
  then
    echo "error: $entry: expected one \`fn main() {}\` to replace with the driver call"
    failed=$((failed + 2))
    return 1
  fi
  cp "$root/bench/slices/driver.rs" "$w/src/bench_driver.rs"
  # Dev features: rustc gets --cfg feature=..., lust gets --features.
  local rustc_cfg=() lust_args=() f
  if [ "$feats" != "-" ]; then
    lust_args+=(--features "$feats")
    local IFS=,
    for f in $feats; do rustc_cfg+=(--cfg "feature=\"$f\""); done
    unset IFS
  fi
  export LUST_CACHE_DIR="$w/cold-cache"
  if run_step rustc-oracle bash -c 'p=$1 e=$2 o=$3; shift 3; rustc --edition 2024 -C overflow-checks=on -A warnings "$@" -o "$p" "$e" && "$p" > "$o"' _ "$w/prog-rustc" "$entry" "$w/rustc.out" "${rustc_cfg[@]}"; then
    # This step runs alone, so the lust build and the program run get the
    # long solo limit directly; a timeout here is a hang and fails the step.
    if run_step lust-build-run bash -c 't=$1 l=$2 e=$3 p=$4 o=$5; shift 5; timeout -k 5 "$t" "$l" build "$e" "$@" -o "$p" && timeout -k 5 "$t" "$p" > "$o"' _ "$retry_secs" "$lust" "$entry" "$w/prog-lust" "$w/lust.out" "${lust_args[@]}"; then
      if ! cmp -s "$w/rustc.out" "$w/lust.out"; then
        passed=$((passed - 1))
        failed=$((failed + 1))
        echo "lust-build-run: FAILED (stdout differs from rustc); diff:"
        diff "$w/rustc.out" "$w/lust.out" | head -10
      fi
    fi
  else
    echo "lust-build-run: skipped (the rustc oracle failed)"
    failed=$((failed + 1))
  fi
  unset LUST_CACHE_DIR
}

if [ $# -ge 1 ]; then
  # Single-slice debug form: cold check plus build and run, no list.
  dir=$1
  name=$(basename "$dir")
  read -r root_name feats _lines <<<"$(slice_meta "$dir")"
  root_name=${root_name:-slice_main.rs}
  feats=${feats:--}
  cold_check "$name" "$dir" "$root_name" "$feats"
  if is_timeout_rc "$(cat "$work/check-$name.rc")"; then
    cold_retry "$name" "$dir" "$root_name" "$feats"
  fi
  if [ -f "$work/check-$name.hang" ]; then
    echo "check $name: FAILED (hang: lust check timed out alone at ${retry_secs}s; first at ${timeout_secs}s)"
    failed=$((failed + 1))
  elif [ "$(cat "$work/check-$name.rc")" = 0 ]; then
    echo "check $name: ok ($(cat "$work/check-$name.ms") ms)"
    passed=$((passed + 1))
  else
    echo "check $name: FAILED ($(cat "$work/check-$name.ms") ms); first lines of the log:"
    head -5 "$work/check-$name.log"
    failed=$((failed + 1))
  fi
  build_and_run "$name" "$dir" "$root_name" "$feats"
  summary
  [ "$failed" -eq 0 ]
  exit $?
fi

# ---- Gate form: every pinned slice. ----
declare -A expected=()
if [ -f "$expected_file" ]; then
  while IFS= read -r line || [ -n "$line" ]; do
    case $line in ''|'#'*) continue ;; esac
    name=${line%%[[:space:]]*}
    reason=${line#"$name"}
    reason=${reason#"${reason%%[![:space:]]*}"}
    expected[$name]=${reason:-no reason recorded}
  done <"$expected_file"
fi

names=() dirs=() roots=() feats_list=()
smallest=""
smallest_lines=""
smallest_i=0
for dir in "$root"/bench/slices/*/; do
  if ! [ -f "$dir/prune.json" ] || ! [ -d "$dir/src" ]; then continue; fi
  name=$(basename "$dir")
  read -r root_name feats lines <<<"$(slice_meta "${dir%/}")"
  [ -n "$root_name" ] || { echo "check $name: FAILED (prune.json has no spec.root)"; failed=$((failed + 1)); continue; }
  names+=("$name"); dirs+=("${dir%/}"); roots+=("$root_name"); feats_list+=("$feats")
  if [ -z "$smallest_lines" ] || [ "${lines:-0}" -lt "$smallest_lines" ]; then
    smallest=$name
    smallest_lines=${lines:-0}
    smallest_i=$(( ${#names[@]} - 1 ))
  fi
done
if [ ${#names[@]} -eq 0 ]; then
  echo "error: no pinned slices found in $root/bench/slices"
  failed=1
  summary
  exit 1
fi
for name in "${!expected[@]}"; do
  case " ${names[*]} " in
    *" $name "*) ;;
    *) echo "expected-fail: FAILED ($name is in $expected_file but is not a pinned slice; remove it from the list)"; failed=$((failed + 1)) ;;
  esac
done

checks_start=$(date +%s%N)
running=0
for i in "${!names[@]}"; do
  cold_check "${names[$i]}" "${dirs[$i]}" "${roots[$i]}" "${feats_list[$i]}" &
  running=$((running + 1))
  if [ "$running" -ge "$jobs" ]; then
    wait -n || true
    running=$((running - 1))
  fi
done
wait
checks_end=$(date +%s%N)

# Solo reruns of timed-out cold checks, before any verdict is read.
for i in "${!names[@]}"; do
  if is_timeout_rc "$(cat "$work/check-${names[$i]}.rc" 2>/dev/null || echo 1)"; then
    cold_retry "${names[$i]}" "${dirs[$i]}" "${roots[$i]}" "${feats_list[$i]}"
  fi
done

for name in "${names[@]}"; do
  rc=$(cat "$work/check-$name.rc" 2>/dev/null || echo 1)
  ms=$(cat "$work/check-$name.ms" 2>/dev/null || echo 0)
  if [ -f "$work/check-$name.hang" ]; then
    # A hang fails even an expected-fail slice: the list records rejects,
    # and a timeout is never a reject.
    echo "check $name: FAILED (hang: lust check timed out alone at ${retry_secs}s; first at ${timeout_secs}s)"
    failed=$((failed + 1))
  elif [ "$rc" = 0 ]; then
    if [ -n "${expected[$name]+x}" ]; then
      echo "check $name: FAILED (passes but is in scripts/slice-check.expected-fail; remove it from the list) ($ms ms)"
      failed=$((failed + 1))
    else
      echo "check $name: ok ($ms ms)"
      passed=$((passed + 1))
    fi
  elif [ -n "${expected[$name]+x}" ]; then
    echo "check $name: expected fail (${expected[$name]}) ($ms ms); first lines of the log:"
    head -3 "$work/check-$name.log"
  else
    echo "check $name: FAILED (exit $rc, $ms ms); first lines of the log:"
    head -5 "$work/check-$name.log"
    failed=$((failed + 1))
  fi
done
echo "cold checks: ${#names[@]} slices in $(( (checks_end - checks_start) / 1000000 )) ms (jobs=$jobs, timeout=${timeout_secs}s per slice)"

if [ -n "${expected[$smallest]+x}" ]; then
  echo "build-run $smallest: skipped (smallest slice is in the expected-fail list)"
else
  build_and_run "$smallest" "${dirs[$smallest_i]}" "${roots[$smallest_i]}" "${feats_list[$smallest_i]}"
fi

wall_end=$(date +%s%N)
echo "slice-check wall: $(( (wall_end - wall_start) / 1000000 )) ms"
summary
[ "$failed" -eq 0 ]
