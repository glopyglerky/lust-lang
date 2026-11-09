#!/usr/bin/env bash
# daemon-soak.sh [N]: many requests through ONE daemon, to catch resource leaks and wedges.
# Mixes builds and runs of distinct small programs (a new session per file), checks and
# builds of an edited crate (one session, many edits), and checks of a few fixed files.
# Every 50 requests it samples the daemon's RSS, fds, threads, children, its temp dir size,
# and the latency of a fixed probe check. Fails when a request hangs (CASE_TIMEOUT,
# default 60 s, retried once alone at RETRY_CASE_TIMEOUT, default 6x; a finished retry
# is "slow", a second timeout is a hang and stops the soak), when a run
# prints the wrong output, when the probe latency at the end is much above the start, or
# when fds, threads, children, RSS, or temp dir size keep growing past fixed bounds.
# Run after scripts/build.sh. N defaults to 1000. Samples: $out/samples.txt (out is printed).
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
lust=$root/target/release/lust
[ -x "$lust" ] || { echo "error: $lust not built. Run scripts/build.sh first."; exit 2; }
N=${1:-1000}
case_timeout=${CASE_TIMEOUT:-60}
# The one timeout rule for client requests (lust_cc). The soak is serial, so
# the retry is alone.
# shellcheck source=scripts/timeout-lib.sh
source "$root/scripts/timeout-lib.sh"
hang=0
unset LUST_NO_DAEMON LUST_CACHE_DIR LUST_DAEMON_IDLE
export LUST_NO_CACHE=1
mkdir -p "${TMPDIR:-/tmp}"
out=$(mktemp -d "${TMPDIR:-/tmp}/lust-soak-XXXXXX")
export TMPDIR=$out/tmp; mkdir -p "$TMPDIR"
export LUST_DAEMON_SOCKET=$out/d.sock
cd "$out" || exit 2
fail=0
bad() { fail=$((fail + 1)); echo "FAIL $*"; }

# prog <i>: a small distinct program; its output is "<i> <sum>".
prog() {
  local i=$1
  cat <<EOF
use std::collections::HashMap;
use std::fmt;
#[derive(Debug, Clone, PartialEq)]
struct P$i { a: i64, b: String }
trait Score { fn score(&self) -> i64; }
impl Score for P$i { fn score(&self) -> i64 { self.a * 3 + self.b.len() as i64 } }
impl fmt::Display for P$i {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}:{}", self.a, self.b) }
}
fn total<T: Score>(xs: &[T]) -> i64 { xs.iter().map(|x| x.score()).sum() }
fn main() {
    let mut m: HashMap<String, P$i> = HashMap::new();
    for k in 0..$((i % 7 + 3)) {
        m.insert(format!("k{k}"), P$i { a: k + $i, b: "x".repeat(k as usize) });
    }
    let mut v: Vec<P$i> = m.into_values().collect();
    v.sort_by_key(|p| p.a);
    let s = total(&v);
    let _ = format!("{} {:?}", v[0], v[0].clone());
    println!("{} {}", $i, s);
}
EOF
}
# expect <i>: the output prog <i> prints.
expect() { local i=$1 n=$(($1 % 7 + 3)) s=0 k; for ((k = 0; k < n; k++)); do s=$((s + (k + i) * 3 + k)); done; echo "$i $s"; }

# edit <j>: the edited crate at version j (body edits, an added fn, and an error every 9th).
edit() {
  local j=$1
  echo "fn helper(x: i64) -> i64 { x * $((j % 13 + 1)) }"
  [ $((j % 4)) = 0 ] && echo "fn extra$j() -> i64 { $j }"
  [ $((j % 9)) = 0 ] && echo "fn broken() -> i64 { \"no\" }"
  echo "fn main() { let v: Vec<i64> = (0..$((j % 5 + 2))).map(helper).collect(); println!(\"{:?}\", v); }"
}

mkdir -p progs fixed
for f in 0 1 2 3; do prog $((100000 + f)) > fixed/f$f.rs; done
probe=fixed/f0.rs
# Start the daemon with the first build; wait for its lock file pid and socket.
# The first build gets the same timeout rule as every later request (req).
lust_cc "$lust" build "$probe" -o fixed/bin0 >/dev/null 2>&1
if is_timeout $?; then echo "daemon-soak: first build timed out alone at ${retry_timeout}s (hang), 1 failed"; exit 1; fi
for _ in $(seq 1 200); do [ -S "$LUST_DAEMON_SOCKET" ] && break; sleep 0.05; done
dpid=$(cat "$LUST_DAEMON_SOCKET.lock" 2>/dev/null)
[ -n "$dpid" ] && [ -d "/proc/$dpid" ] || { echo "daemon-soak: no daemon started, 1 failed"; exit 1; }
echo "soak: daemon pid $dpid, $N requests, dir $out"

# req <cmd...>: one client request under the timeout rule (lust_cc). Sets rc.
# A second timeout is a hang: it fails here and sets $hang so the main loop
# stops instead of queueing more requests behind a wedged daemon.
req() {
  lust_cc "$lust" "$@" >"$out/last.out" 2>"$out/last.err"; rc=$?
  if is_timeout "$rc"; then
    hang=1
    bad "HANG request timed out alone at ${retry_timeout}s (first at ${case_timeout}s): $*"
  elif [ "$lust_cc_slow" = 1 ]; then
    echo "slow: request ($*) timed out at ${case_timeout}s, finished alone at ${retry_timeout}s"
  fi
}
ms() { date +%s%N | cut -c1-13; }
probe_ms() { local t0; t0=$(ms); req check "$probe"; echo $(($(ms) - t0)); }
sample() {
  local i=$1 rss fds thr kids tmpkb lat
  rss=$(awk '/^VmRSS/ {print $2}' "/proc/$dpid/status" 2>/dev/null)
  fds=$(ls "/proc/$dpid/fd" 2>/dev/null | wc -l)
  thr=$(awk '/^Threads/ {print $2}' "/proc/$dpid/status" 2>/dev/null)
  kids=$(ps --ppid "$dpid" --no-headers 2>/dev/null | wc -l)
  tmpkb=$(du -sk "$TMPDIR" 2>/dev/null | cut -f1)
  lat=$(probe_ms); lat2=$(probe_ms); lat3=$(probe_ms)
  lat=$(printf '%s\n' "$lat" "$lat2" "$lat3" | sort -n | sed -n 2p)
  echo "$i rss_kb=${rss:-0} fds=$fds threads=${thr:-0} children=$kids tmp_kb=${tmpkb:-0} probe_ms=$lat" | tee -a samples.txt
}

: > samples.txt
sample 0
t_start=$(ms)
for ((i = 1; i <= N; i++)); do
  [ -d "/proc/$dpid" ] || { bad "daemon $dpid died at request $i"; break; }
  case $((i % 10)) in
    0|1|2|3)  # distinct program, build then run its binary
      prog "$i" > progs/p$i.rs
      req build progs/p$i.rs -o progs/b$i
      if [ $rc = 0 ]; then
        got=$(timeout 10 progs/b$i 2>&1); [ "$got" = "$(expect "$i")" ] || bad "p$i printed '$got', want '$(expect "$i")'"
      else bad "p$i build rc $rc: $(head -c 300 "$out/last.err")"; fi
      rm -f progs/b$i ;;
    4)  # distinct program through `lust run`
      prog "$i" > progs/p$i.rs
      req run progs/p$i.rs
      [ "$(cat "$out/last.out")" = "$(expect "$i")" ] || bad "run p$i printed '$(cat "$out/last.out")' rc $rc" ;;
    5|6|7)  # one crate, many edits: check, then build every other time
      edit "$i" > edit.rs
      if [ $((i % 2)) = 0 ]; then req check edit.rs; else req build edit.rs -o edit.bin; fi
      want=0; [ $((i % 9)) = 0 ] && want=1
      if [ "$want" = 0 ] && [ $rc != 0 ]; then bad "edit $i rc $rc: $(head -c 300 "$out/last.err")"; fi
      if [ "$want" = 1 ] && [ $rc = 0 ]; then bad "edit $i accepted a type error"; fi ;;
    8|9)  # fixed files: warm checks and builds
      f=fixed/f$((i % 4)).rs
      if [ $((i % 3)) = 0 ]; then req build "$f" -o fixed/bin; else req check "$f"; fi
      [ $rc = 0 ] || bad "fixed $f rc $rc" ;;
  esac
  [ "$hang" = 1 ] && { echo "soak: stopping after a hang"; break; }
  [ $((i % 50)) = 0 ] && sample "$i"
  [ "$hang" = 1 ] && { echo "soak: stopping after a hang"; break; }
  [ "$fail" -gt 20 ] && { echo "soak: too many failures, stopping"; break; }
done
echo "soak: $N requests in $(( ($(ms) - t_start) / 1000 )) s"

# Bounds. Latency: the probe median of the last 3 samples within 3x (+20 ms) of the first 3.
# fds, threads, children: the end within a small margin of the start. RSS and temp dir: the
# end within 2x of the sample at request 200 (+ a floor), so a slow leak fails.
read_col() { awk -v k="$1" '{for (i = 2; i <= NF; i++) { split($i, a, "="); if (a[1] == k) print a[2] }}' samples.txt; }
med() { sort -n | awk '{a[NR] = $1} END {print a[int((NR + 1) / 2)]}'; }
first_lat=$(read_col probe_ms | sed -n 2,4p | med)
last_lat=$(read_col probe_ms | tail -3 | med)
[ "$last_lat" -le $((first_lat * 3 + 20)) ] || bad "probe latency grew: start ${first_lat} ms, end ${last_lat} ms"
for k in fds threads children; do
  s=$(read_col $k | sed -n 2p); e=$(read_col $k | tail -1)
  [ "$e" -le $((s + 8)) ] || bad "$k grew: $s -> $e"
done
for k in rss_kb tmp_kb; do
  s=$(read_col $k | sed -n 5p); e=$(read_col $k | tail -1)
  [ -n "$s" ] || s=$(read_col $k | sed -n 2p)
  [ "$e" -le $((s * 2 + 65536)) ] || bad "$k grew: $s at request 200 -> $e at the end"
done
# Stop the daemon and wait for it: on exit it sweeps its own temp dir, which races
# with the rm below.
kill "$dpid" 2>/dev/null
for _ in $(seq 1 100); do [ -d "/proc/$dpid" ] || break; sleep 0.05; done
# The daemon logs a stats line every 100 requests (sessions, expansion tables).
grep -h "stats:" fixed/.lust-cache/daemon.log 2>/dev/null | sed -n '1p;$p'
if [ "$fail" = 0 ]; then
  cd / && rm -rf "$out" 2>/dev/null
  echo "daemon-soak: $N requests, probe ${first_lat} -> ${last_lat} ms, 0 failed"
  exit 0
else
  echo "samples: $out/samples.txt"
  echo "daemon-soak: $N requests, $fail failed"; exit 1
fi
