#!/usr/bin/env bash
# speed-row.sh [slice-dir]: the quick speed row for the landing gate (non-blocking).
# Runs the slice benchmark in --quick mode (default: bench/slices/goport-util-small)
# and prints one line per row plus a `speed-row:` summary line with the warm body
# edit ratio. Then scripts/tsedit.py runs one probe of each edit kind on the same slice
# (body, infer, sig, sigx, oblig, derive, generic, rpit; each with a reject state) and
# the summary line ends with the ratio per kind. Exit 1 only when a benchmark itself
# fails (lust and rustc disagree, warm and cold lust answers differ, a hang, or lust
# rejects the slice); speed never fails it.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
slice=${1:-$root/bench/slices/goport-util-small}
out=${TMPDIR:-/tmp}/speed-row-$$.json
edits=${TMPDIR:-/tmp}/speed-row-tsedit-$$.json
python3 "$root/scripts/scale-gate.py" --slice "$slice" --quick --json "$out" || exit 1
python3 "$root/scripts/tsedit.py" --slice "$slice" --probes 1 --json "$edits" | tail -12
rc=${PIPESTATUS[0]}
python3 - "$out" "$edits" <<'PY'
import json, os, sys
d = json.load(open(sys.argv[1]))
row = {r["row"]: r for r in d["rows"]}
b = row["body edit check (one fn)"]
c = row["cold check"]
pk = b.get("patch_kept", {}).get("patch", {})
patch = f", body patch {pk['hits']} hit / {pk['misses']} miss" if pk else ""
kinds = ""
if os.path.exists(sys.argv[2]):
    e = json.load(open(sys.argv[2]))
    rs = [r for r in e["rows"] if r.get("ratio")]
    bad = sum(r["false_accepts"] + r["warm_ne_cold"] + r["hangs"] for r in e["rows"])
    kinds = ("; edit kinds vs cargo check (median): " + ", ".join(f"{r['kind']} {r['ratio']}x" for r in rs)
             + (f", worst {min(rs, key=lambda r: r['ratio'])['kind']}" if rs else "")
             + (f", {bad} SOUNDNESS FAILURES" if bad else ""))
print(f"speed-row: {d['slice']} {d['lines']} lines: warm body-edit check lust {b['lust']['median']:.0f} ms "
      f"vs rustc {b['rustc_best']['median']:.0f} ms ({b['ratio']}x median, {b.get('ratio_p95')}x p95, target 100x{patch}); "
      f"cold check {c['lust']['median']:.0f} vs {c['rustc_best']['median']:.0f} ms ({c['ratio']}x){kinds}")
PY
rm -f "$out" "$edits"
exit $rc
