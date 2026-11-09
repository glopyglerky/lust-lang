#!/bin/sh
# Build the shim and run every test against both build variants.
#
# - Cargo builds liblust_shim.so twice: default (system allocator)
#   and --features jemalloc.
# - tests/test_shim.c links through the pinned tcc CLI against the
#   cdylib (the path spikes/RESULTS.md K6-4 proves), and once through
#   the system cc.
# - src/bin/oracle.rs does the probe operations directly in Rust.
#   The ORACLE lines of the C runs must match it byte for byte.
#
# Exit status is 0 only when every run prints RESULT PASS and every
# ORACLE diff is empty. Sizes and times print at the end.
set -eu

cd "$(dirname "$0")"
ROOT=$(cd .. && pwd)

# Find the pinned tcc: env first, then the repo third_party, then the
# main worktree of this repo (worktrees do not carry third_party).
TCC=""
if [ -n "${LUST_TCC_HOME:-}" ] && [ -x "$LUST_TCC_HOME/bin/tcc" ]; then
    TCC="$LUST_TCC_HOME/bin/tcc"
elif [ -x "$ROOT/third_party/tcc/bin/tcc" ]; then
    TCC="$ROOT/third_party/tcc/bin/tcc"
else
    MAIN=$(git worktree list --porcelain 2>/dev/null | sed -n 's/^worktree //p' | head -n 1)
    if [ -n "$MAIN" ] && [ -x "$MAIN/third_party/tcc/bin/tcc" ]; then
        TCC="$MAIN/third_party/tcc/bin/tcc"
    fi
fi
if [ -z "$TCC" ]; then
    echo "pinned tcc not found; run scripts/build-tcc.sh or set LUST_TCC_HOME" >&2
    exit 1
fi

OUT="$PWD/out"
rm -rf "$OUT"
mkdir -p "$OUT/default" "$OUT/jemalloc" "$OUT/fixture"

now_ms() {
    date +%s%3N
}

echo "== build default shim (cargo, release)"
t0=$(now_ms)
cargo build --release
BUILD_DEFAULT_MS=$(( $(now_ms) - t0 ))
cp target/release/liblust_shim.so "$OUT/default/liblust_shim.so"
cp target/release/lust-shim-oracle "$OUT/oracle"

echo "== build jemalloc shim (cargo, release, --features jemalloc)"
t0=$(now_ms)
CARGO_TARGET_DIR="$PWD/target-jemalloc" cargo build --release --features jemalloc --lib
BUILD_JEMALLOC_MS=$(( $(now_ms) - t0 ))
cp target-jemalloc/release/liblust_shim.so "$OUT/jemalloc/liblust_shim.so"

# Fixtures: a stat file with fixed bytes and mtime, a zone file.
STATFILE="$OUT/fixture/statfile.txt"
printf 'lust shim stat probe\n' > "$STATFILE"
touch -d @1700000000 "$STATFILE"
chmod 644 "$STATFILE"

if [ -f /usr/share/zoneinfo/America/New_York ]; then
    TZIF=/usr/share/zoneinfo/America/New_York
    ZONE=America/New_York
else
    TZIF=/usr/share/zoneinfo/UTC
    ZONE=UTC
fi
export TZ="$ZONE"

CC=${CC:-cc}
echo "== link C test with pinned tcc: $TCC"
t0=$(now_ms)
"$TCC" -w -Iinclude tests/test_shim.c -L"$OUT/default" -llust_shim \
    -Wl,-rpath="$OUT/default" -lpthread -lm -ldl -o "$OUT/test_default"
LINK_TCC_MS=$(( $(now_ms) - t0 ))
"$TCC" -w -Iinclude tests/test_shim.c -L"$OUT/jemalloc" -llust_shim \
    -Wl,-rpath="$OUT/jemalloc" -lpthread -lm -ldl -o "$OUT/test_jemalloc"

echo "== link C test with system cc (header and rpath control)"
t0=$(now_ms)
"$CC" -O1 -Wall -Wextra -Iinclude tests/test_shim.c -L"$OUT/default" -llust_shim \
    -Wl,-rpath="$OUT/default" -lpthread -o "$OUT/test_cc"
LINK_CC_MS=$(( $(now_ms) - t0 ))

echo "== oracle (direct Rust)"
"$OUT/oracle" "$TZIF" "$ZONE" "$STATFILE" > "$OUT/oracle_all.txt"
grep '^ORACLE' "$OUT/oracle_all.txt" > "$OUT/oracle.txt"
wc -l < "$OUT/oracle.txt"

run_and_check() {
    name=$1
    bin=$2
    echo "== run $name"
    rm -rf "$OUT/work-$name"
    mkdir -p "$OUT/work-$name"
    "$bin" "$TZIF" "$ZONE" "$STATFILE" "$OUT/work-$name" > "$OUT/$name.txt" 2>&1 || {
        echo "$name exited non-zero; output:" >&2
        cat "$OUT/$name.txt" >&2
        return 1
    }
    cat "$OUT/$name.txt"
    if grep -q 'MUST_NOT_APPEAR' "$OUT/$name.txt"; then
        echo "$name leaked a dup2 child write" >&2
        return 1
    fi
    grep -q '^RESULT PASS' "$OUT/$name.txt" || {
        echo "$name did not pass" >&2
        return 1
    }
    grep '^ORACLE' "$OUT/$name.txt" > "$OUT/$name.oracle.txt"
    if diff -u "$OUT/oracle.txt" "$OUT/$name.oracle.txt" > "$OUT/$name.diff"; then
        echo "PASS oracle-diff-$name"
    else
        echo "FAIL oracle-diff-$name" >&2
        cat "$OUT/$name.diff" >&2
        return 1
    fi
    return 0
}

run_and_check c_default "$OUT/test_default"
run_and_check c_cc "$OUT/test_cc"
run_and_check c_jemalloc "$OUT/test_jemalloc"

echo "== sizes and times"
for v in default jemalloc; do
    SIZE=$(stat -c %s "$OUT/$v/liblust_shim.so")
    cp "$OUT/$v/liblust_shim.so" "$OUT/stripped.so"
    strip "$OUT/stripped.so"
    STRIPPED=$(stat -c %s "$OUT/stripped.so")
    echo "liblust_shim.so ($v): $SIZE bytes, stripped $STRIPPED bytes"
done
echo "cargo build default: ${BUILD_DEFAULT_MS} ms (cache state as found)"
echo "cargo build jemalloc: ${BUILD_JEMALLOC_MS} ms (separate target dir, cold first run)"
echo "tcc link of C test (default): ${LINK_TCC_MS} ms"
echo "cc link of C test (default): ${LINK_CC_MS} ms"
echo "PASS all shim tests"
