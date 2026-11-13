#!/usr/bin/env bash
# Parser robustness fuzz: truncations and mutations of tests/pass files, plus random
# token soup. Every case runs under a timeout (PARSE_FUZZ_TIMEOUT seconds, default 2). A hang (timeout), a compiler
# panic (exit 101), or a signal death (exit > 128) is a failure; clean accepts and
# clean errors are both fine. Failing inputs are kept in a directory printed at the end.
# Rule-zero sample: up to PARSE_FUZZ_RUSTC (default 60; 0 turns it off) of the cases lust
# accepts are built with default-lint rustc (scripts/rustc_sample.py); a rustc rejection is
# bad too. The run prints a "parse-fuzz: rustc sample: ..." line before its summary.
# Usage: scripts/parse-fuzz.sh [cases] [seed]
set -uo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
lust=$root/target/release/lust
cases=${1:-3000}
seed=${2:-1}
work=$(mktemp -d)
faildir=$(mktemp -d)
echo "parse-fuzz: $cases cases, seed $seed, work dir $work"

python3 - "$root" "$work" "$cases" "$seed" <<'PY'
import glob, os, random, sys
root, work, cases, seed = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
rng = random.Random(seed)
sources = [open(f, "rb").read() for f in sorted(glob.glob(os.path.join(root, "tests/pass/*.rs")))]
tokens = [b"fn", b"let", b"mut", b"struct", b"enum", b"impl", b"match", b"if", b"else",
          b"loop", b"while", b"for", b"in", b"return", b"break", b"pub", b"pub(",
          b"use", b"gen", b"r#gen", b"x", b"main", b"Vec", b"i64", b"self", b"Self",
          b"(", b")", b"{", b"}", b"[", b"]", b"<", b">", b">>", b";", b",", b":",
          b"::", b".", b"..", b"..=", b"->", b"=>", b"=", b"==", b"+", b"-", b"*",
          b"/", b"%", b"&", b"&&", b"|", b"||", b"!", b"#", b"#[derive(", b"?",
          b"$", b"@", b"_", b"0", b"128i8", b"0x", b"1e", b"1.5", b"'a", b"'\\n'",
          b"'", b'"s"', b'"unterminated', b"/*", b"*/", b"// c\n", b"println!",
          b"18446744073709551616u64", "🧀".encode()]
known = [b"(", b"", b"pub(", b"fn", b"{", b"'", b'"', b"/*", b"r#", b"r#self",
         b"fn main() { let x = ", b"####", b"<<<<", b"fn f(&mut self,,,,",
         b"struct S {", b"match x {", b"#[",
         b"fn main() { let x = " + b"(" * 5000 + b"1",
         b"fn main() { let x: i64 = 1" + b" + 1" * 5000 + b"; }",
         b"fn main() { let x: i64 = " + b"-" * 5000 + b"1; }",
         # fx-parsehang: an error-typed argument against a blanket `&T` impl took minutes.
         b"fn f<P: AsRef<std::path::Path>>(p: P) {} fn main() { " + b"f(&target); " * 16 + b"}"]

def write(i, data):
    with open(os.path.join(work, "case-%05d.rs" % i), "wb") as f:
        f.write(data)

for i in range(cases):
    if i < len(known):
        write(i, known[i])
        continue
    mode = i % 3
    src = rng.choice(sources)
    if mode == 0:  # truncation at a random byte offset
        write(i, src[: rng.randint(0, len(src))])
    elif mode == 1:  # random token soup
        n = rng.randint(1, 80)
        write(i, b" ".join(rng.choice(tokens) for _ in range(n)))
    else:  # a few random mutations of a real file
        data = bytearray(src)
        for _ in range(rng.randint(1, 5)):
            if not data:
                break
            op = rng.randint(0, 2)
            pos = rng.randint(0, len(data))
            if op == 0:
                end = min(len(data), pos + rng.randint(1, 60))
                del data[pos:end]
            elif op == 1:
                data[pos:pos] = rng.choice(tokens)
            else:
                end = min(len(data), pos + rng.randint(1, 60))
                data[pos:pos] = data[pos:end]
        write(i, bytes(data))
PY

# Run from the temp dir so no lust daemon for this repo answers instead of a cold check,
# and do not autostart a daemon for the temp dir either.
export LUST_BIN="$lust" FAILDIR="$faildir"
run_one() {
  f="$1"
  LUST_NO_DAEMON=1 timeout "${PARSE_FUZZ_TIMEOUT:-2}" "$LUST_BIN" check "$f" >/dev/null 2>&1
  code=$?
  if [ "$code" -eq 0 ]; then
    echo "$f" >> "$FAILDIR/.accepted"
  elif [ "$code" -eq 124 ]; then
    # A timeout under parallel load is retried once later, alone, with 6x the limit; only a
    # second timeout is a hang (two land gates timed out on cases that pass in under 10 s idle).
    echo "$f" >> "$FAILDIR/.slow"
  elif [ "$code" -eq 101 ] || [ "$code" -gt 128 ]; then
    cp "$f" "$FAILDIR/"
    echo "BAD exit=$code $f"
  fi
}
export -f run_one
cd "$work" || exit 1
ls case-*.rs | xargs -P 8 -I{} bash -c 'run_one "$@"' _ {}
if [ -s "$faildir/.slow" ]; then
  while read -r f; do
    LUST_NO_DAEMON=1 timeout $(( ${PARSE_FUZZ_TIMEOUT:-2} * 6 )) "$lust" check "$f" >/dev/null 2>&1
    code=$?
    if [ "$code" -eq 124 ] || [ "$code" -eq 101 ] || [ "$code" -gt 128 ]; then
      cp "$f" "$faildir/"; echo "BAD exit=$code $f (hang: also timed out alone at 6x the limit)"
    else
      echo "slow: $f timed out under load, passed alone"
      [ "$code" -eq 0 ] && echo "$f" >> "$faildir/.accepted"
    fi
  done < "$faildir/.slow"
  rm -f "$faildir/.slow"
fi

rbad=0
if [ "${PARSE_FUZZ_RUSTC:-60}" -gt 0 ]; then
  touch "$faildir/.accepted"
  # shellcheck disable=SC2046 # one path per line, no spaces (case-NNNNN.rs)
  python3 "$root/scripts/rustc_sample.py" "${PARSE_FUZZ_RUSTC:-60}" "$seed" "$faildir" $(cat "$faildir/.accepted") \
    | sed 's/^/parse-fuzz: /'
  rbad=$(find "$faildir" -name 'rustc-reject-*.rs' | wc -l)
fi
rm -f "$faildir/.accepted"
bad=$(( $(find "$faildir" -name 'case-*.rs' | wc -l) + rbad ))
echo "parse-fuzz: $cases cases, $bad bad"
if [ "$bad" -ne 0 ]; then
  echo "failing inputs kept in $faildir"
  exit 1
fi
rm -rf "$work" "$faildir"
