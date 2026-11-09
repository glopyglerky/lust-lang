#!/usr/bin/env bash
# fetch-deps.sh [--check] [deps-file...]: puts every registry crate this tree can ask Cargo for
# into the local Cargo cache (~/.cargo/registry), for every platform. The gate runs Cargo with
# CARGO_NET_OFFLINE=true, so a crate missing from the sandbox snapshot fails a gate.
#
# Sources, each its own Cargo package (exact pins of one crate can differ between them):
#   - every Cargo.toml outside target dirs (lust, tools/lust-port, library/deps, tests, bench), with its lock
#   - every tests/adapters/crates/*/deps.toml (scripts/test-adapters.sh)
#   - every `// cargo-dep:` header set in tools/lust-port/tests/features (scripts/port-scan-sync.py)
#   - each deps-file given: [dependencies] lines, as in a deps.toml (crates of unlanded branches)
# `cargo fetch` without --target gets the packages of all platforms. Cargo metadata needs them
# (windows-link on Linux) unless it is given --filter-platform.
#
# --check: runs `cargo metadata` (unfiltered, so all platforms; what the probes' Cargo runs need)
# with CARGO_NET_OFFLINE=true instead. It fails, naming the package, when a crate is not in the
# cache. Offline `cargo fetch` can pass where metadata fails. Run it on a sandbox from a new
# snapshot before the gate uses it.
# Works on a copy of the tree: Cargo writes a Cargo.lock next to a manifest that has none.
# Exit: 0 when every package fetched, 1 when one failed.
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
check=0
[ "${1:-}" = --check ] && { check=1; shift; }
cargo=(cargo fetch -q)
[ $check = 1 ] && { export CARGO_NET_OFFLINE=true; cargo=(cargo metadata -q --format-version 1); }
work=$(mktemp -d "${TMPDIR:-/tmp}/fetch-deps.XXXXXX")
trap 'rm -rf "$work"' EXIT
tar -C "$root" --exclude=./.git --exclude=target --exclude=boat-out -cf - . | tar -xf - -C "$work" ||
  { echo "fetch-deps: copying the tree failed"; exit 1; }
mkdir -p "$work/.sets"

# add_set <name> <file of [dependencies] lines>: one synthetic package per distinct set.
add_set() {
  local id
  id=$(sort -u "$2" | sha1sum | cut -c1-12)
  [ -d "$work/.sets/$id" ] && return
  mkdir -p "$work/.sets/$id/src"
  echo 'fn main() {}' >"$work/.sets/$id/src/main.rs"
  { printf '# %s\n[package]\nname = "fetch-set"\nversion = "0.0.0"\nedition = "2024"\n\n[dependencies]\n' "$1"
    cat "$2"; printf '\n[workspace]\n'; } >"$work/.sets/$id/Cargo.toml"
}
for f in "$work"/tests/adapters/crates/*/deps.toml; do
  [ -f "$f" ] && add_set "${f#"$work"/}" "$f"
done
for f in "$work"/tools/lust-port/tests/features/*.rs; do
  sed -n 's|^//[[:space:]]*cargo-dep:[[:space:]]*||p' "$f" >"$work/.sets/lines"
  [ -s "$work/.sets/lines" ] && add_set "${f#"$work"/}" "$work/.sets/lines"
done
for f in "$@"; do
  [ -f "$f" ] || { echo "fetch-deps: no such deps file: $f"; exit 1; }
  add_set "$f" "$f"
done

ok=0 bad=0
while IFS= read -r m; do
  # A fixture that Cargo itself rejects (a missing path dep on purpose) has nothing to fetch.
  if out=$(cd "$(dirname "$m")" && "${cargo[@]}" --manifest-path "$m" 2>&1 >/dev/null); then
    ok=$((ok + 1))
  elif grep -qiE 'download|offline|http|registry|index' <<<"$out"; then
    bad=$((bad + 1)); echo "FAIL ${m#"$work"/}: $(grep -m1 -iE 'error' <<<"$out")"
    case $m in "$work"/.sets/*) echo "  from: $(head -1 "$m" | cut -c3-)";; esac
  else
    echo "skip ${m#"$work"/}: $(grep -m1 -iE 'error' <<<"$out")"
  fi
done < <(find "$work" -name Cargo.toml -not -path '*/target/*' | sort)
echo "fetch-deps: $ok fetched, $bad failed$([ $check = 1 ] && echo ' (offline check)')"
[ $bad = 0 ]
