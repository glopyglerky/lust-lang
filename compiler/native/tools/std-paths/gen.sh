#!/usr/bin/env bash
# gen.sh <out file>: regenerates src/resolve/std_paths.txt for the
# rustc on PATH, with rustc as the oracle. Needs network once
# (rustup component add rust-src). Heavy: run it on a boat sandbox,
#   /tmp/lust-wt/boat-run.sh --fetch out -- 'bash tools/std-paths/gen.sh out/std_paths.txt'
# (about 30 minutes on 16 cores).
#  1. pool.py: every identifier in the std, core, alloc, and core_arch
#     source (pool.txt), and the names it declares pub (pubnames.txt).
#  2. gen1.py + parse1.py: rustc's "consider importing" suggestions for
#     every pool name, in std, no_std, and no_std + alloc crates: the
#     importable names (step1.txt).
#  3. walk.py: every module from std, core, and alloc, walked with a
#     glob import and one `use self::Name as _;` probe per name.
#  4. traits.py: marks the traits (they take `{self}` and `{}`).
#  5. compact.py: the table format src/resolve/stdpaths.rs reads.
set -euo pipefail
out=$(realpath -m "$1")
here=$(cd "$(dirname "$0")" && pwd)
rustup component add rust-src >/dev/null 2>&1
src=$(rustc --print sysroot)/lib/rustlib/src/rust/library
work=$(mktemp -d)
cd "$work"
cp "$here"/*.py .
python3 pool.py "$src"
mkdir -p s1
for mode in std core alloc; do for k in $(seq 0 39); do echo "$mode $k"; done; done |
  xargs -P12 -L1 sh -c 'python3 gen1.py pool.txt $0 $1 40 > s1/$0-$1.rs; rustc --edition 2024 --error-format=json --crate-type lib --emit=metadata -o s1/$0-$1.rmeta s1/$0-$1.rs 2>&1 | python3 parse1.py > s1/$0-$1.out'
cat s1/*.out | sort -u > step1.txt
tr ':' '\n' < step1.txt | grep -v '^$' | sort -u > step1-names.txt
python3 walk.py step1-names.txt pubnames.txt
python3 traits.py table.txt
mkdir -p "$(dirname "$out")"
python3 compact.py table.txt "$out"
