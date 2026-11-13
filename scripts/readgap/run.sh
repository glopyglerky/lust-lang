#!/usr/bin/env bash
# run.sh [SEED] [COUNT]: the read-gap lane. Generates COUNT histories from SEED
# (scripts/readgap/gen.py) and runs them through scripts/test-daemon-scenarios.py in daemon
# mode, in front-image (restart) mode, and in front-image mode with release C (CI's build:
# per-slot C and unit objects reused across builds): warm vs cold vs rustc on accept or reject
# and on run stdout. Exit 0 when every mode passes. LUST_BIN as for the scenario runner.
set -u
seed=${1:-1}
count=${2:-34}
root=$(cd "$(dirname "$0")/../.." && pwd)
out=$(mktemp -d "${TMPDIR:-/tmp}/readgap-XXXXXX")
trap 'rm -rf "$out"' EXIT
python3 "$root/scripts/readgap/gen.py" --seed "$seed" --count "$count" --out "$out" >/dev/null || exit 2
dirs=("$out"/*)
rc=0
for mode in daemon image release; do
    unset DAEMON_SCENARIO_RESTART LUST_RELEASE_C
    [ $mode = daemon ] || export DAEMON_SCENARIO_RESTART=1
    [ $mode = release ] && export LUST_RELEASE_C=1
    log=$(python3 "$root/scripts/test-daemon-scenarios.py" "${dirs[@]}" 2>&1)
    r=$?
    grep -E "^FAIL" <<<"$log" | sed "s/^/readgap $mode: /"
    echo "readgap $mode (seed $seed, $count histories): $(tail -1 <<<"$log")"
    [ $r = 0 ] || rc=1
done
exit $rc
