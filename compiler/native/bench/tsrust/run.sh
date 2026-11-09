#!/usr/bin/env bash
# Reruns the ts-rust rustc baseline on a COPY of ts-rust. OPT-IN: a full pass
# takes tens of minutes. The checked-in numbers live in baseline.json; this
# script exists so they can be refreshed, and so a lust column can slot in
# next to the rustc column when lust can build the crate.
#
#   TSRUST_RERUN=1 bench/tsrust/run.sh [work-dir]
#
# Env:
#   TSRUST_SRC   ts-rust checkout to copy (default /home/theo/.t3/projects/ts-rust)
#   TSRUST_ROWS  space-separated subset: cold editloop (default "cold editloop")
#
# The original checkout is only read (rsync source). All builds and edits
# happen in the copy. The goport-profile row (708 s) is never rerun here.
# Results land in <work-dir>/results.jsonl (timeit.py format, same schema as
# the original baseline run) and <work-dir>/lust.json.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
src=${TSRUST_SRC:-/home/theo/.t3/projects/ts-rust}
work=${1:-${TMPDIR:-/tmp}/lust-tsrust-rerun}
rows=${TSRUST_ROWS:-"cold editloop"}

if [ "${TSRUST_RERUN:-0}" != 1 ]; then
  echo "opt-in only: set TSRUST_RERUN=1 (a full pass takes tens of minutes)."
  echo "checked-in numbers: bench/tsrust/baseline.json"
  exit 0
fi
[ "$(cd "$src" && pwd)" != "$work" ] || { echo "work dir must differ from the source checkout"; exit 1; }

copy=$work/src
export CARGO_TARGET_DIR=$work/target
mkdir -p "$work/logs"
results=$work/results.jsonl
timeit() { python3 "$here/timeit.py" "$results" "$@"; }

echo "copying $src -> $copy"
rm -rf "$copy"
mkdir -p "$copy"
rsync -a --exclude target --exclude .git "$src/" "$copy/"
python3 "$here/edits.py" "$copy" --check
cd "$copy"

want() { case " $rows " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }

if want cold; then
  # Fully fresh check and build (deps included), stable default dev.
  cargo clean --quiet 2>/dev/null || true
  timeit fresh-full-check-stable 1 "$work/logs/check.log" -- cargo check -p ts_goport --locked
  timeit fresh-full-debug-build-stable 1 "$work/logs/build.log" -- cargo build -p ts_goport --locked
  # Crate-only cold, best stable config: bootstrap threads=8, debuginfo 0.
  for op in check build; do
    cargo clean -p ts_goport -p goport_util -p goport_lsproto --quiet
    RUSTC_BOOTSTRAP=1 RUSTFLAGS="-Z threads=8 -C debuginfo=0" \
      timeit "crate-only-$op-threads8" 1 "$work/logs/$op-threads8.log" -- cargo "$op" -p ts_goport --locked
  done
fi

if want editloop; then
  # Prime incremental artifacts, then the paired edits e1..e5fix.
  cargo check -p ts_goport --locked >/dev/null 2>&1
  cargo build -p ts_goport --locked >/dev/null 2>&1
  for e in e1 e2 e3 e4 e5 e5fix; do
    python3 "$here/edits.py" "$copy" "$e"
    timeit "edit-$e-check" 1 "$work/logs/edit-$e.log" -- cargo check -p ts_goport --locked
    if [ "$e" != e5 ]; then
      timeit "edit-$e-build" 1 "$work/logs/edit-$e.log" -- cargo build -p ts_goport --locked
    fi
  done
fi

# The lust column. lust compiles single .rs files today; it cannot build a
# cargo workspace. Record that fact in the same shape baseline.json uses, so
# filling the column in later is a data change, not a harness rewrite. When
# lust gains workspace builds, replace this block with paired runs: same
# copy, same edits.py sequence, `lust check`/`lust build` per step.
lust_bin=$root/target/release/lust
smallest=$(find "$copy/crates/ts_goport/src" -name '*.rs' | head -1)
probe="not run"
if [ -x "$lust_bin" ] && [ -n "$smallest" ]; then
  probe=$("$lust_bin" check "$smallest" 2>&1 | head -1 || true)
fi
python3 - "$work/lust.json" "$probe" <<'PY'
import json, sys
json.dump({
    "lust_s": None,
    "lust_note": "lust cannot build a cargo workspace yet; column reserved",
    "probe_first_line": sys.argv[2],
}, open(sys.argv[1], "w"), indent=2)
PY
echo "done. results: $results"
echo "compare against bench/tsrust/baseline.json before updating any number."
