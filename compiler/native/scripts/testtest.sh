#!/usr/bin/env bash
# Differential tests for `lust test` against `rustc --test` (the oracle).
# For every tests/testrun/*.rs file, each #[test] must get the same result under both:
# pass, fail, timeout, or ignored. Tests run one at a time on both sides, so one bad test
# cannot hide the results of the others. The rustc side uses the default panic=unwind
# test harness (rustc does not allow panic=abort with --test); a panic is a fail on
# both sides, so the per-test results are comparable.
# Usage: scripts/testtest.sh [filter]
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
lust=$root/target/release/lust
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
filter=${1:-}
pass=0
fail=0

report() {
  if [ "$1" = ok ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL $2: $3"; fi
}

command -v timeout >/dev/null || { echo "testtest: the `timeout` command is required"; exit 1; }

for f in "$root"/tests/testrun/*.rs; do
  name=$(basename "$f" .rs)
  [[ -n "$filter" && "$name" != *"$filter"* ]] && continue
  if ! rustc --edition 2024 -C overflow-checks=on -A warnings --test -o "$work/$name.rt" "$f" 2>"$work/$name.rt.err"; then
    report bad "$name" "rustc --test rejected it: $(head -3 "$work/$name.rt.err")"; continue
  fi
  # Expected results from rustc, one run per test.
  : > "$work/$name.expected"
  "$work/$name.rt" --list 2>/dev/null | sed -n 's/: test$//p' > "$work/$name.names"
  while IFS= read -r t; do
    timeout 1 "$work/$name.rt" --exact "$t" >"$work/$name.one.out" 2>&1
    rc=$?
    # An ignored test exits 0 without running; its line names it.
    if grep -q "test $t ... ignored" "$work/$name.one.out"; then
      echo "$t ignored"
    else
      case $rc in
        0) echo "$t pass" ;;
        124) echo "$t timeout" ;;
        *) echo "$t fail" ;;
      esac
    fi >> "$work/$name.expected"
  done < "$work/$name.names"
  # Actual results from lust.
  "$lust" test "$f" --json --timeout 1 > "$work/$name.lust.json" 2>"$work/$name.lust.err"
  lust_exit=$?
  if python3 - "$work/$name.expected" "$work/$name.lust.json" "$lust_exit" <<'PYEOF'
import json, sys
expected = {}
for line in open(sys.argv[1]):
    parts = line.split()
    if len(parts) == 2:
        expected[parts[0]] = parts[1]
actual, summary = {}, None
for line in open(sys.argv[2]):
    rec = json.loads(line)
    if rec.get("kind") == "test":
        actual[rec["name"]] = rec["status"]
        assert isinstance(rec["duration_ms"], (int, float)), "duration_ms must be a number"
        assert "stdout" in rec and "stderr" in rec, "case must carry captured output"
    elif rec.get("kind") == "summary":
        summary = rec
errors = []
if summary is None:
    errors.append("no summary record")
elif summary["build"] != "ok":
    errors.append(f"summary build is {summary['build']}")
if actual != expected:
    errors.append(f"rustc={expected} lust={actual}")
want_exit = 0 if all(v in ("pass", "ignored") for v in expected.values()) else 101
if int(sys.argv[3]) != want_exit:
    errors.append(f"exit code {sys.argv[3]}, want {want_exit}")
if summary is not None:
    counts = {s: list(actual.values()).count(s) for s in ("pass", "fail", "timeout")}
    got = (summary["passed"], summary["failed"], summary["timed_out"], summary["total"])
    want = (counts["pass"], counts["fail"], counts["timeout"], len(actual))
    if got != want:
        errors.append(f"summary counts {got}, want {want}")
if errors:
    print("; ".join(errors))
    sys.exit(1)
PYEOF
  then
    report ok "$name"
  else
    report bad "$name" "lust test differs from rustc --test (see above)"
    continue
  fi
  # Filter parity: the filter is a substring, so it must run exactly the tests whose
  # names contain the first test's name.
  first=$(head -1 "$work/$name.names")
  if [ -n "$first" ]; then
    want=$(grep -c "$first" "$work/$name.names")
    got=$("$lust" test "$f" "$first" --json --timeout 1 2>/dev/null | grep -c '"kind":"test"')
    if [ "$got" = "$want" ]; then report ok "$name/filter"; else report bad "$name/filter" "filter ran $got tests, want $want"; fi
  fi
done

echo "testtest: $pass passed, $fail failed"
[ $fail -eq 0 ]
