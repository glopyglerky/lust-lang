#!/usr/bin/env bash
# CI gate: the daemon must give the same answer as a cold compile (see the equivalence
# rule at the top of fuzz.py), with no crashes, no hangs, and the fresh-session guard
# never firing. Exits nonzero on any failure.
# Usage: scripts/fuzz/run.sh --edits N --seed S [--crate small|small2|small3|realish|multi|multi2|multi3|patchseq] [--modules M]
#        [--strict] [-- <extra fuzz.py args>]
set -uo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
edits=2000
seed=7
crate=small
modules=40
strict=()
extra=()
while [ $# -gt 0 ]; do
  case "$1" in
    --edits) edits=$2; shift 2 ;;
    --seed) seed=$2; shift 2 ;;
    --crate) crate=$2; shift 2 ;;
    --modules) modules=$2; shift 2 ;;
    --strict) strict=(--strict); shift ;;
    --) shift; extra=("$@"); break ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
if [ ! -x "$root/target/release/lust" ]; then
  echo "target/release/lust missing; run: cargo build --release" >&2
  exit 2
fi
out=${LUST_FUZZ_OUT:-/tmp/lust-fuzz-out}
exec python3 "$root/scripts/fuzz/fuzz.py" run --edits "$edits" --seed "$seed" \
  --crate "$crate" --modules "$modules" --out "$out" --fail-on-guard "${strict[@]}" "${extra[@]}"
