#!/usr/bin/env bash
# Checks the diagnostic code registry (docs/errors/REGISTRY.md) against the tree:
#   1. Registry ranges are well formed and do not overlap.
#   2. Every L#### code in src/ is inside an "in use" registry range. A code in a
#      "reserved" range, or outside every range, fails.
#   3. Every "in use" code is emitted in src/, has a page in docs/errors/, and has
#      a row in src/explain.rs. Every page and every explain row is "in use".
# No build is needed. Usage: scripts/check-codes.sh [root]
set -uo pipefail
root=${1:-$(cd "$(dirname "$0")/.." && pwd)}
registry=$root/docs/errors/REGISTRY.md
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
fail=0
bad() { fail=$((fail + 1)); echo "FAIL $1: $2"; }

[ -f "$registry" ] || { bad registry "missing $registry"; echo "check-codes: 1 failed"; exit 1; }

# Parse table rows "| L0501-L0509 | area | owner | reserved | notes |" into
# "lo hi status" lines (lo/hi as plain numbers). Malformed code rows are errors.
awk -F'|' '
  function trim(s) { gsub(/^ +| +$/, "", s); return s }
  /^\| *L[0-9]/ {
    range = trim($2); status = trim($5)
    if (range !~ /^L[0-9]{4}(-L[0-9]{4})?$/) { print "bad-range " range > "/dev/stderr"; err = 1; next }
    if (status != "in use" && status != "reserved") { print "bad-status " range " " status > "/dev/stderr"; err = 1; next }
    lo = substr(range, 2, 4) + 0
    hi = (length(range) > 5) ? substr(range, 8, 4) + 0 : lo
    if (hi < lo) { print "bad-order " range > "/dev/stderr"; err = 1; next }
    print lo, hi, (status == "in use" ? "inuse" : "reserved")
  }
  END { exit err }
' "$registry" > "$work/ranges" 2> "$work/parse-errors"
if [ -s "$work/parse-errors" ]; then
  while IFS= read -r line; do bad registry "$line"; done < "$work/parse-errors"
fi
[ -s "$work/ranges" ] || bad registry "no code rows found in $registry"

sort -n "$work/ranges" > "$work/ranges-sorted"

# 1. No two rows claim the same code: sorted ranges must not overlap.
awk '
  NR > 1 && $1 <= prev_hi { printf "overlap L%04d-L%04d vs L%04d-L%04d\n", prev_lo, prev_hi, $1, $2 }
  { prev_lo = $1; prev_hi = $2 }
' "$work/ranges-sorted" > "$work/overlaps"
while IFS= read -r line; do [ -n "$line" ] && bad registry "$line"; done < "$work/overlaps"

# Expand ranges to one code per line.
expand() { # $1 = status to keep (or "all"), writes codes to stdout
  awk -v want="$1" 'want == "all" || $3 == want { for (n = $1; n <= $2; n++) printf "L%04d\n", n }' "$work/ranges-sorted" | sort -u
}
expand inuse > "$work/inuse"
expand reserved > "$work/reserved"
expand all > "$work/registered"

# Code sets from the tree, the same grep check-explain.sh uses.
grep -rhoE 'L[0-9]{4}' "$root"/src/ | sort -u > "$work/emitted"
ls "$root"/docs/errors/L*.md 2>/dev/null | sed 's|.*/||; s|\.md||' | sort -u > "$work/pages"
grep -oE '\("L[0-9]{4}"' "$root/src/explain.rs" | grep -oE 'L[0-9]{4}' | sort -u > "$work/wired"

# Reports one failure per code in "codes in file $1 but not in file $2".
only_in() { # $1 = left file, $2 = right file, $3 = check name, $4 = message
  comm -23 "$1" "$2" > "$work/only"
  while IFS= read -r code; do [ -n "$code" ] && bad "$3/$code" "$4"; done < "$work/only"
}

# 2. Emitted codes must be registered and in use.
only_in "$work/emitted" "$work/registered" range "emitted in src/ but outside every range in docs/errors/REGISTRY.md; add a reserved range first"
comm -12 "$work/emitted" "$work/reserved" > "$work/emitted-reserved"
while IFS= read -r code; do
  [ -n "$code" ] && bad "reserved/$code" "emitted in src/ but the registry says reserved; flip it to in use and add its explain page"
done < "$work/emitted-reserved"

# 3. In use, pages, and explain rows must agree.
only_in "$work/inuse" "$work/emitted" stale "registry says in use but the code is not in src/"
only_in "$work/inuse" "$work/pages" page "in use but has no page in docs/errors/"
only_in "$work/pages" "$work/inuse" page "page exists but the code is not in use in the registry"
only_in "$work/inuse" "$work/wired" explain "in use but has no row in src/explain.rs"
only_in "$work/wired" "$work/inuse" explain "row in src/explain.rs but the code is not in use in the registry"

inuse_n=$(wc -l < "$work/inuse")
reserved_n=$(wc -l < "$work/reserved")
echo "check-codes: $inuse_n codes in use, $reserved_n reserved, $fail failed"
[ $fail -eq 0 ]
