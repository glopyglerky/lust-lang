#!/usr/bin/env bash
# lust vs rustc on the same generated file, with matched settings.
#
# Honesty rules:
#   - Same flags for both compilers where a flag has a lust counterpart:
#     edition 2024, panic=abort, overflow checks on.
#   - Debug info is matched per row. A "debuginfo=0" row runs rustc with
#     -C debuginfo=0 against `lust build`; a "debuginfo=2" row runs rustc with
#     -C debuginfo=2 against `lust build -g`. (The old script compared
#     `lust build` against rustc debuginfo=2 times. That row is gone.)
#   - rustc gets its fastest valid configuration per row. We time four
#     configurations and report the best median, with its name:
#     default, -Z threads=8 (RUSTC_BOOTSTRAP=1), incremental, threads+incremental.
#   - Times are wall time of the whole process (start to exit).
#   - Runs are interleaved: each round runs every competitor once, in rotating
#     order, so machine load drift hits all of them. We report median and p90.
#   - "cold" means a fresh incremental cache and a fresh output file. The OS
#     page cache is warm (one untimed warm-up run per competitor).
#   - "after edit" applies a one-line edit before each timed run. Incremental
#     configurations keep their cache across edits (the file alternates between
#     the original and edited text, so every timed run follows a real edit).
#     lust runs here without a daemon, so its "after edit" is a cold compile;
#     the warm daemon edit loop is bench/realish/agent_loop.py.
#
# Usage: bench/compare.sh <shape: plain|idiomatic> <units>...   (about 15 lines per unit)
# Env: REPS (default 3) timed rounds per competitor.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
lust=$root/target/release/lust
shape=$1
shift
reps=${REPS:-3}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/samples"
# lust must compile cold in this script: no daemon autostart, no disk cache.
# (Warm numbers live in bench/realish/agent_loop.py and bench/run.py.)
export LUST_NO_DAEMON=1 LUST_NO_CACHE=1 LUST_DAEMON_SOCKET="$work/no-daemon.sock"
base_flags=(--edition 2024 -C panic=abort -C overflow-checks=on -A warnings)

# now_ns: nanoseconds since epoch. GNU date has %N; BSD date (macOS) does not,
# so fall back to python3 there (adds the same few ms to every competitor).
if [ "$(date +%N 2>/dev/null | tr -d 0-9)" = "" ]; then
  now_ns() { date +%s%N; }
else
  now_ns() { python3 -c 'import time; print(time.time_ns())'; }
fi

# time_run <sample-file> <cmd...>: run cmd, append wall seconds to sample-file.
time_run() {
  local samples=$1
  shift
  local s e
  s=$(now_ns)
  "$@" >/dev/null 2>"$work/last.err"
  e=$(now_ns)
  awk -v s="$s" -v e="$e" 'BEGIN { printf "%.4f\n", (e - s) / 1000000000 }' >>"$samples"
}

# stats <sample-file>: print "median p90".
stats() {
  sort -g "$1" | awk '{ v[NR] = $1 } END {
    m = v[int((NR + 1) / 2)]
    p = v[int(NR * 0.9 + 0.999)]
    printf "%.3f %.3f", m, p
  }'
}

# rustc_cmd <config> <kind> <debuginfo> <src> <out>: build the rustc argv in CMD.
rustc_cmd() {
  local config=$1 kind=$2 debuginfo=$3 src=$4 out=$5
  CMD=(rustc "${base_flags[@]}" -C "debuginfo=$debuginfo")
  case $config in
    *threads*) CMD=(env RUSTC_BOOTSTRAP=1 "${CMD[@]}" -Z threads=8) ;;
  esac
  case $config in
    *inc*) CMD+=(-C "incremental=$work/inc-$kind$debuginfo-$config") ;;
  esac
  if [ "$kind" = check ]; then
    CMD+=(--emit=metadata -o "$out.rmeta" "$src")
  else
    CMD+=(-o "$out" "$src")
  fi
}

# run_competitor <tag> <kind> <debuginfo> <mode> <rep>: one run of one competitor.
# tag is a rustc config name or "lust". Timed unless TIMED=0.
run_competitor() {
  local tag=$1 kind=$2 debuginfo=$3 mode=$4 rep=$5
  local src=$work/main.rs out=$work/out-$tag-$kind$debuginfo
  # Pick the source text for this run. Cold uses the original text. Edit mode
  # alternates edited, original, edited, ...; the incremental caches were
  # primed with the original text (a cold run), so every timed edit-mode run
  # follows a real one-line edit.
  if [ "$mode" = edit ] && [ $((rep % 2)) -eq 0 ]; then
    cp "$work/edit.rs" "$src"
  else
    cp "$work/orig.rs" "$src"
  fi
  if [ "$tag" = lust ]; then
    if [ "$kind" = check ]; then
      CMD=("$lust" check "$src")
    elif [ "$debuginfo" = 2 ]; then
      CMD=("$lust" build -g "$src" -o "$out")
    else
      CMD=("$lust" build "$src" -o "$out")
    fi
  else
    # A cold run must not reuse an incremental cache.
    if [ "$mode" = cold ]; then
      rm -rf "$work/inc-$kind$debuginfo-$tag"
    fi
    rustc_cmd "$tag" "$kind" "$debuginfo" "$src" "$out"
  fi
  if [ "${TIMED:-1}" = 1 ]; then
    time_run "$work/samples/$kind$debuginfo-$mode-$tag" "${CMD[@]}"
  else
    "${CMD[@]}" >/dev/null 2>"$work/last.err"
  fi
}

configs=(default threads8 inc threads8+inc)

# bench_row <label> <kind> <debuginfo> <mode>: time one table row, print it.
bench_row() {
  local label=$1 kind=$2 debuginfo=$3 mode=$4
  local tags=("${configs[@]}" lust)
  # Prime incremental caches for the after-edit rows with the original text.
  if [ "$mode" = edit ]; then
    local t
    for t in inc threads8+inc; do
      rm -rf "$work/inc-$kind$debuginfo-$t"
      TIMED=0 run_competitor "$t" "$kind" "$debuginfo" cold 0
    done
  fi
  local rep i tag
  for ((rep = 0; rep < reps; rep++)); do
    for ((i = 0; i < ${#tags[@]}; i++)); do
      tag=${tags[$(((i + rep) % ${#tags[@]}))]}
      run_competitor "$tag" "$kind" "$debuginfo" "$mode" "$rep"
    done
  done
  # Best rustc configuration by median.
  local best_tag="" best_med="" med p90 lmed lp90
  for tag in "${configs[@]}"; do
    read -r med p90 <<<"$(stats "$work/samples/$kind$debuginfo-$mode-$tag")"
    if [ -z "$best_med" ] || awk -v a="$med" -v b="$best_med" 'BEGIN { exit !(a < b) }'; then
      best_tag=$tag best_med=$med
    fi
  done
  read -r med p90 <<<"$(stats "$work/samples/$kind$debuginfo-$mode-$best_tag")"
  read -r lmed lp90 <<<"$(stats "$work/samples/$kind$debuginfo-$mode-lust")"
  printf "| %-9s | %6s | %-26s | %-13s | %8.3fs | %8.3fs | %8.3fs | %8.3fs | %6.1fx |\n" \
    "$shape" "$lines" "$label" "$best_tag" "$med" "$p90" "$lmed" "$lp90" \
    "$(awk -v a="$med" -v b="$lmed" 'BEGIN { print a / b }')"
}

printf "| %-9s | %6s | %-26s | %-13s | %9s | %9s | %9s | %9s | %7s |\n" \
  shape lines operation "rustc best" "rustc med" "rustc p90" "lust med" "lust p90" speedup
printf "|%s|\n" "-----------|--------|----------------------------|---------------|-----------|-----------|-----------|-----------|---------"
for units in "$@"; do
  python3 "$root/bench/rustc-baseline/gen.py" "$shape" "$units" 0 >"$work/orig.rs"
  python3 "$root/bench/rustc-baseline/gen.py" "$shape" "$units" 1 >"$work/edit.rs"
  lines=$(wc -l <"$work/orig.rs" | tr -d ' ')

  # One untimed warm-up per competitor (page cache, first-touch costs).
  for tag in "${configs[@]}" lust; do
    TIMED=0 run_competitor "$tag" check 0 cold 0
  done
  rm -rf "$work"/inc-*

  bench_row "check cold (debuginfo=0)" check 0 cold
  bench_row "check after edit (debuginfo=0)" check 0 edit
  bench_row "build cold (debuginfo=0)" build 0 cold
  bench_row "build after edit (debuginfo=0)" build 0 edit
  bench_row "build cold (debuginfo=2)" build 2 cold
  bench_row "build after edit (debuginfo=2)" build 2 edit

  # Same program, same output (debuginfo=0 builds of the original text).
  cp "$work/orig.rs" "$work/main.rs"
  rustc "${base_flags[@]}" -C debuginfo=0 -o "$work/prog-rustc" "$work/main.rs"
  "$lust" build "$work/main.rs" -o "$work/prog-lust"
  "$work/prog-rustc" >"$work/r.out"
  "$work/prog-lust" >"$work/l.out"
  same=$(cmp -s "$work/r.out" "$work/l.out" && echo yes || echo NO)
  echo "|           |        | same output: $same |"
  [ "$same" = yes ]
done
