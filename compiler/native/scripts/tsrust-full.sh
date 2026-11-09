#!/usr/bin/env bash
# tsrust-full.sh [--boat] [--out DIR] [census options]: compile the whole ts-rust fork with lust.
#
# The fork is the submodule third_party/ts-rust (docs/PLAN.md, BRIEF "The ts-rust fork").
# Steps: get the fork tree, build and test it with Cargo, `lust check` then `lust build` every
# lib, bin and cdylib target, and print the errors grouped by root cause. The last line is
#   tsrust-full: <ok>/<targets> check, <ok>/<bins> build, <k> root causes
# Census options go to scripts/tsrust_full.py: --skip-cargo, --skip-build, --only a,b, -j N, --build-j N,
# --timeout SECS, --parity (scripts/tsrust_parity.py after the builds). Results: <out>/results.json, <out>/cargo.log, <out>/bin/ (lust builds).
# Default out: $TMPDIR/tsrust-full (/tmp/tsrust-full when TMPDIR is unset).
#
# Fork tree, first source that works:
#   1. third_party/ts-rust as it is (a checked-out submodule; local edits are used as they are)
#   2. ~/up/ts-rust.tgz (a boat sandbox: --boat uploads it)
#   3. a clone from the local reference (TSRUST_REF, default /home/theo/.t3/projects/ts-rust)
#      when it has the pinned commit, so no network; else from the fork's remote, with
#      the reference for objects
#
# --deep: run scripts/tsrust_deep.py instead (lines kept out per root cause; its options:
# --only a,b, --max-rounds N). Results: <out>/deep.json and a ranked table.
# --build: run scripts/tsrust_build.py instead (the census build: prune the fork with cfg(test) on,
# then lust build and Cargo build of each lib's unit tests as one bin, and compare every test's
# run; its options: --only a,b, --max-rounds N, --timeout SECS, -j N). Results: <out>/build.json.
# --tsedit: run scripts/tsedit.py instead (warm one-fn edit kinds, lust daemon vs cargo check,
# on the census clean fork; its options: --target a,b (default: the 3 libs and ts_wasm),
# --clean TGZ (default: auto, the --rustc census first), --kinds, --probes, --quick, --no-cold).
# Lust is a release build there (timing). Results: <out>/tsedit.json, <out>/tsedit-<target>.json.
# --boat: run all of this on a boat sandbox (/tmp/lust-wt/boat-run.sh, BOAT_RUN overrides).
# boat-run.sh does not pack submodule trees, so only this job uploads the fork (one tar of its
# tracked and untracked files, no target/). Results come back to ./boat-out/tsrust-full/
# (--deep: ./boat-out/tsrust-full-deep/, --build: ./boat-out/tsrust-full-build/). TSRUST_BOAT_HOURS sets the time limit (default 3).
# LUST=<path> runs another lust build (a census of a branch before it lands).
# Needs: cargo and the fork's crates in the cargo registry (offline), lust built (scripts/build.sh).
set -uo pipefail
# The lust checkout: this script's parent dir (on a boat sandbox ~/w is not a git repo).
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
sub=third_party/ts-rust
boat=0; deep=""; tsedit=""; out=${TMPDIR:-/tmp}/tsrust-full; args=()
while [ $# -gt 0 ]; do
  case "$1" in
    --boat) boat=1; shift;;
    --deep) deep=--deep; shift;;
    --build) deep=--build; shift;;
    --tsedit) tsedit=--tsedit; shift;;
    --out) out=$2; shift 2;;
    *) args+=("$1"); shift;;
  esac
done

# fork_tree: make $sub hold the fork. Prints where it came from.
fork_tree() {
  local want ref
  if [ -f "$HOME/up/ts-rust.tgz" ] && [ ! -f "$sub/Cargo.toml" ]; then
    mkdir -p "$sub" && tar -xzf "$HOME/up/ts-rust.tgz" -C "$sub" || return 1
    echo "tsrust-full: fork tree: ~/up/ts-rust.tgz ($(cat "$sub/.fork-head" 2>/dev/null))"; return 0
  fi
  want=$(git ls-files -s "$sub" | awk '{print $2}')
  [ -n "$want" ] || { echo "tsrust-full: $sub is not a submodule here" >&2; return 1; }
  if [ -f "$sub/Cargo.toml" ]; then
    local have; have=$(git -C "$sub" rev-parse HEAD 2>/dev/null || echo "unknown")
    [ "$have" = "$want" ] || echo "tsrust-full: note: fork tree at $have, lust pins $want"
    echo "tsrust-full: fork tree: $sub ($have)"; return 0
  fi
  ref=${TSRUST_REF:-/home/theo/.t3/projects/ts-rust}
  if git -C "$ref" cat-file -e "$want^{commit}" 2>/dev/null; then
    # Clone the reference itself: no network. origin then points at the fork's remote.
    rmdir "$sub" 2>/dev/null
    git clone -q --no-checkout --reference "$ref" "$ref" "$sub" && git -C "$sub" checkout -q "$want" || return 1
    git -C "$sub" remote set-url origin "$(git config -f .gitmodules submodule.$sub.url)"
    echo "tsrust-full: fork tree: clone of $ref at $want"; return 0
  fi
  git submodule update --init --reference "$ref" "$sub" || return 1
  echo "tsrust-full: fork tree: fetched $want from the fork remote"
}

if [ $boat = 1 ]; then
  fork_tree || exit 2
  W=$(mktemp -d "${TMPDIR:-/tmp}/tsrust-boat.XXXXXX")
  git -C "$sub" rev-parse HEAD > "$W/.fork-head" 2>/dev/null
  (cd "$sub" && git ls-files -co --exclude-standard -z) > "$W/files" || exit 2
  tar -czf "$W/ts-rust.tgz" -C "$sub" --null -T "$W/files" -C "$W" .fork-head || exit 2
  q=""; for x in "${args[@]}"; do q+=" $(printf %q "$x")"; done
  name=${TSRUST_BOAT_NAME:-tsrust-full${deep:+-${deep#--}}${tsedit:+-tsedit}}
  [ "$deep" = --build ] && q+=" --clean"  # also the build census's bins and run dirs
  # tsedit times lust: a release build (BRIEF "Building lust"); the others use the gate profile.
  build="scripts/build.sh"
  [ -n "$tsedit" ] && build="cargo build --release -q"
  # The fetch skips the cargo target dirs, the lust cache and the deep census's fork copy (several
  # GB on /tmp); it keeps results, logs and the lust-built bins.
  "${BOAT_RUN:-/tmp/lust-wt/boat-run.sh}" --hours "${TSRUST_BOAT_HOURS:-3}" --up "$W/ts-rust.tgz" --fetch "$name" -- \
    "$build >/dev/null 2>&1 || { $build 2>&1 | tail -30; exit 8; }; scripts/tsrust-full.sh $deep $tsedit --out ~/w/$name$q; rc=\$?;" \
    "rm -rf ~/w/$name/cargo-target ~/w/$name/rustc-target ~/w/$name/lust-cargo-target ~/w/$name/lust-cache ~/w/$name/fork ~/w/$name/census/{cargo-target,rustc-target,lust-cache,fork}; exit \$rc"
  rc=$?; rm -rf "$W"; exit $rc
fi

fork_tree || exit 2
lust=${LUST:-$root/target/release/lust}
[ -x "$lust" ] || { echo "tsrust-full: no $lust; run scripts/build.sh first" >&2; exit 2; }
export LUST_TCC_HOME=${LUST_TCC_HOME:-/home/theo/.t3/projects/lust-lang/third_party/tcc}
mkdir -p "$out"
[ "$deep" = --build ] && exec python3 "$root/scripts/tsrust_build.py" --fork "$root/$sub" --lust "$lust" --work "$out" "${args[@]}"
[ -n "$deep" ] && exec python3 "$root/scripts/tsrust_deep.py" --fork "$root/$sub" --lust "$lust" --work "$out" "${args[@]}"
if [ -n "$tsedit" ]; then
  have_target=0; have_clean=0
  for x in "${args[@]}"; do [ "$x" = --target ] && have_target=1; [ "$x" = --clean ] && have_clean=1; done
  [ $have_target = 1 ] || args+=(--target goport_util,goport_lsproto,ts_goport,ts_wasm)
  [ $have_clean = 1 ] || args+=(--clean auto --census-only goport_util,goport_lsproto,ts_goport,ts_wasm)
  exec python3 "$root/scripts/tsedit.py" --fork "$root/$sub" --lust "$lust" --out "$out" --json "$out/tsedit.json" "${args[@]}"
fi
exec python3 "$root/scripts/tsrust_full.py" --fork "$root/$sub" --lust "$lust" --out "$out" "${args[@]}"
