#!/usr/bin/env bash
# Builds lust with the fast gate profile (Cargo.toml [profile.gate]) and copies `lust` and
# `lust-core` to target/release/, where the test scripts look for them. Use it for test runs;
# use `cargo build --release` to time lust. Extra args go to cargo (e.g. --all-targets).
# Note: after this, target/release/ may hold gate binaries until the next release build that
# has a source change to rebuild.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
cargo build --profile gate "$@"
mkdir -p target/release
# New files (rm first): cargo hardlinks target/release/lust* to target/release/deps/.
rm -f target/release/lust target/release/lust-core
cp target/gate/lust target/gate/lust-core target/release/
