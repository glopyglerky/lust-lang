#!/usr/bin/env bash
# `lust run` tests, cold (no daemon). Small programs (one function unit) must run in memory
# (see tcc::run_in_memory). That run forks only when the OS lists one thread in the process,
# so the in-memory check is also the thread count check at every fork point. The programs
# must behave exactly like the built executable: same stdout, same
# stderr (panic text included), same exit status. Both must also match rustc on stdout and
# exit status. Programs run with two extra arguments, so `std::env::args().count()` counts
# them. Usage: scripts/runtest.sh [filter]
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
lust=$root/target/release/lust
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# Make sure no daemon answers (and none autostarts) and the disk cache stays out: the
# client must exec lust-core, which compiles cold and runs in memory.
export LUST_DAEMON_SOCKET="$work/no-daemon.sock"
export LUST_NO_DAEMON=1
export LUST_NO_CACHE=1
filter=${1:-}
# Cases run in parallel; rustc's results come from the cache (scripts/case-pool.sh).
oracle_args=(x y)
source "$root/scripts/case-pool.sh"

# runtest_case <name> <file>: one tests/pass case, in its own dir.
runtest_case() {
  local name=$1 f=$2 rs bs ms probe mode want_mode
  rustc_oracle run "$f" --edition 2024 -C overflow-checks=on -A warnings
  [ "$(cat rustc.cc)" -eq 0 ] || { report bad "$name" "rustc rejected it"; return; }
  run_lust "$lust" build "$f" -o "$name.lust" 2>"$name.build.err" || { report bad "$name" "lust build rejected it"; return; }
  rs=$(cat rustc.rs)
  run_to_err "$name.lust.err" "./$name.lust" x y >"$name.lust.out"; bs=$?
  run_lust "$lust" run "$f" -- x y >"$name.mem.out" 2>"$name.mem.err"; ms=$?
  # Which run path did it take? A program with more than one function unit (glue_shared has
  # three; the glue_*_type_cross_unit tests force a split with filler fns; a program that
  # splices a std source module has hundreds of functions) must use the file. Units compiled
  # = main, glue, runtime, table, loc, plus the function units.
  probe=$(run_lust "$lust" run --timings "$f" -- x y 2>&1 >/dev/null)
  if grep -q "in memory" <<<"$probe"; then mode=mem; else mode=file; fi
  units=$(run_lust "$lust" build --timings "$f" -o "$name.units" 2>&1 >/dev/null | grep -o '[0-9]* units compiled' | grep -o '^[0-9]*')
  want_mode=mem; [ "${units:-0}" -gt 6 ] && want_mode=file
  # The glue_*_type_cross_unit tests force a unit split with filler fns.
  [[ "$name" == glue_shared || "$name" == glue_*_type_cross_unit ]] && want_mode=file
  # A program that names a std platform module (W-SHIM) compiles the
  # platform crate's fns too, often more than one unit: either path is fine.
  grep -oE 'std::(io|fs|env|path|process|ffi)(::[a-z_]+)?' "$f" | grep -vqxE 'std::env::args|std::process::(exit|abort)' && want_mode=$mode
  if [ "$mode" != "$want_mode" ]; then
    report bad "$name" "run took the $mode path, want $want_mode"
  elif [ $ms -ne $bs ] || [ $ms -ne $rs ]; then
    report bad "$name" "exit status memory=$ms built=$bs rustc=$rs"
  elif ! cmp -s <(mask_stdout "$f" "$name.mem.out") <(mask_stdout "$f" "$name.lust.out") || ! cmp -s <(mask_stdout "$f" "$name.mem.out") rustc.out; then
    report bad "$name" "stdout differs (memory vs built vs rustc)"
  elif ! cmp -s <(mask_stderr "$name.mem.err") <(mask_stderr "$name.lust.err"); then
    report bad "$name" "stderr differs (memory vs built)"; diff <(mask_stderr "$name.mem.err") <(mask_stderr "$name.lust.err") | head -5
  else
    report ok "$name"
  fi
}
run_all runtest_case "$root"/tests/pass/*.rs

echo "runtest: $pass passed, $fail failed"
[ $fail -eq 0 ]
