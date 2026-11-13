#!/usr/bin/env bash
# release-x64.sh <ts-rust-checkout> <out-dir>: one warm linux-x64 release
# build of tsc (x86_64-unknown-linux-musl, static) with the lust release
# driver, then the caller's verification. State that makes it warm, all
# restored by the caller (CI cache, replay harness) before the call:
#   $STATE_DIR    rel-front's front-end image (LUST_FRONT_STATE); used
#                 only when FRONT=1 and the lust build supports it
#   $CACHE_DIR    lust's disk cache (cc objects; LUST_CACHE_DIR)
# Verification: when GOLDENS=<file> and GOLDENS_PY=<goldens.py> are set,
# the 216 tsgo fixture runs of the built tsc compare against the
# digests in GOLDENS (made from a Cargo tsc of the same source state).
# Prints one phase line per step and a final total (seconds).
set -uo pipefail
SRC=$1 OUT=$2
LUST=${LUST:-lust}
STATE_DIR=${STATE_DIR:-$OUT/state}
CACHE_DIR=${CACHE_DIR:-$OUT/cache}
mkdir -p "$OUT" "$STATE_DIR" "$CACHE_DIR"
now() { date +%s.%N; }
span() { awk -v a="$1" -v b="$2" 'BEGIN{printf "%.2f", b-a}'; }
T0=$(now)

ENVS=(LUST_NO_DAEMON=1 LUST_CACHE_DIR="$CACHE_DIR")
[ "${FRONT:-0}" = 1 ] && ENVS+=(LUST_FRONT_STATE="$STATE_DIR")
[ "${RELC:-0}" = 1 ] && ENVS+=(LUST_RELEASE_C=1)
T1=$(now)
env "${ENVS[@]}" "$LUST" build "$SRC/crates/ts_goport/src/bin/tsgo.rs" \
  --manifest-path "$SRC/crates/ts_goport/Cargo.toml" --bin tsgo --offline \
  --no-default-features --features noembed --profile goport \
  --target x86_64-unknown-linux-musl -o "$OUT/tsc" --timings > "$OUT/build.log" 2>&1
RC=$?
T2=$(now)
echo "build rc=$RC $(span $T1 $T2)s"
[ $RC = 0 ] || { tail -n 20 "$OUT/build.log"; exit $RC; }
grep -m1 "cc compile" "$OUT/build.log" || true
grep -o "cc x86_64-unknown-linux-musl: [^)]*)" "$OUT/build.log" | head -1 || true

cp "$SRC"/crates/ts_goport/libs/lib*.d.ts "$OUT"/
T3=$(now)
VER="skipped"
if [ -n "${GOLDENS:-}" ]; then
  ln -sf tsc "$OUT/tsgo"
  python3 "$GOLDENS_PY" check "$SRC" "$OUT/tsgo" "$GOLDENS" "$OUT/verify-work" > "$OUT/verify.log" 2>&1
  VRC=$?
  VER=$(tail -n 1 "$OUT/verify.log")
  echo "verify rc=$VRC $VER"
  [ $VRC = 0 ] || exit $VRC
fi
T4=$(now)
file "$OUT/tsc" | grep -q "statically linked" && ST=static || ST=DYNAMIC
echo "artifact $OUT/tsc $ST $(stat -c %s "$OUT/tsc") bytes, version $("$OUT/tsc" --version)"
echo "total $(span $T0 $T4)s (build $(span $T1 $T2), verify $(span $T3 $T4))"
