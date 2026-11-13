#!/usr/bin/env bash
# Regenerates src/runtime/unicode_table.h from the rustc on PATH (1.99,
# Unicode 17). Run after a rustc update; the header records the version.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
rustc --edition 2024 -O -o "$work/gen_unicode" "$root/tools/gen_unicode.rs"
"$work/gen_unicode" "$(rustc --version)" > "$work/unicode_table.h"
mv "$work/unicode_table.h" "$root/src/runtime/unicode_table.h"
echo "wrote $root/src/runtime/unicode_table.h"
