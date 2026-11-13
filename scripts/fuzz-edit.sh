#!/usr/bin/env bash
# Short run of the daemon-vs-cold random-edit fuzzer for CI (scripts/ci.sh calls this
# with FUZZ_SECONDS, default 20). The full gate is scripts/fuzz/run.sh; see the
# equivalence rule at the top of scripts/fuzz/fuzz.py.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
secs=${FUZZ_SECONDS:-20}
# A small-crate edit costs about 10 ms. Keep most of the budget for fuzzing and
# leave a few seconds for the wire and libbase smoke tests.
edits=$(( secs * 60 ))
[ "$edits" -lt 200 ] && edits=200
"$root/scripts/fuzz/run.sh" --edits "$edits" --seed 7 --crate small
python3 "$root/scripts/fuzz/smoke.py"
