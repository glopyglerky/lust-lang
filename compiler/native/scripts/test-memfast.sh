#!/usr/bin/env bash
# test-memfast.sh: the release runtime's vector mem* (src/runtime/memfast.c) and memmem
# prefilter (src/runtime/deps.c) against byte-by-byte references (tests/runtime/memfast_test.c),
# with ASan and UBSan under each C compiler found (clang, gcc), plus the word path
# (-DMF_NO_V16) that non-x86_64 targets take; and runtime.h's lust_vec_fits against the
# division it replaced (tests/runtime/vecfits_test.c), with tcc too. Exit 1 on any failure.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT
rc=0
for cc in clang gcc; do
  command -v "$cc" > /dev/null || continue
  for v in "" "-DMF_NO_V16"; do
    "$cc" -O1 -g $v -fsanitize=address,undefined -fno-sanitize-recover=undefined -fno-strict-aliasing -fsigned-char -w \
      tests/runtime/memfast_test.c -o "$out/t" || { echo "test-memfast: $cc $v: build failed"; rc=1; continue; }
    "$out/t" | sed "s/^/test-memfast ($cc $v): /" || rc=1
  done
done
tcc=${LUST_TCC_BIN:-${LUST_TCC_HOME:+$LUST_TCC_HOME/bin/tcc}}
[ -n "$tcc" ] || tcc=third_party/tcc/bin/tcc
for cc in clang gcc "$tcc"; do
  command -v "$cc" > /dev/null || continue
  f="-O1 -w"; [ "$cc" = "$tcc" ] || f="$f -fsanitize=undefined -fno-sanitize-recover=undefined"
  "$cc" $f tests/runtime/vecfits_test.c -o "$out/v" || { echo "test-memfast: $cc vecfits: build failed"; rc=1; continue; }
  "$out/v" | sed "s|^|test-memfast ($cc): |" || rc=1
done
exit $rc
