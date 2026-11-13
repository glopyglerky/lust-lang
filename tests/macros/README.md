# Macro test corpus

This is the test corpus for W-MACRO (ARCH section 6). The gate is
`scripts/test-macros.sh`. Run it before you change macro work.

## Layout

| Path | What |
|---|---|
| `pass/*.rs` | Executable fixtures. Each prints observable results. |
| `pass/*.out` | The exact stdout of the rustc build. |
| `pass/*.expanded` | Canonical tokens of the expansion (see normalize.py), space-joined and wrapped. |
| `reject/*.rs` | Programs rustc rejects. The header records the rustc error class. |
| `ts-rust-names.txt` | The 70 ts-rust macro names the corpus must cover. |
| `normalize.py` | Turns expanded output into the token form (ARCH 6.5: compare tokens, not text). |
| `check_origin.py` | Checks copied macro text against the ts-rust tree. |
| `pass/node_scaffold.rs.inc` | Shared Node stand-ins, `include!`d by the three node fixtures. Not a test. |
| `data/`, `libs/` | Files read by `include_str!` and `include_bytes!` fixtures. |

Fixture name prefixes: `ts_` holds a macro definition copied from
ts-rust. `sem_` is a semantics case from ARCH 6.2 and 6.3 or the
architecture reviews. `rej_` is a both-reject case.

## The three layers

1. Token compare. The runner expands each pass fixture with
   `RUSTC_BOOTSTRAP=1 rustc -Zunpretty=expanded`, normalizes the
   output to tokens, and diffs it with the stored `.expanded` file.
2. Executable cases. The runner builds each pass fixture with
   rustc (edition 2024, overflow checks on), runs it, and diffs
   stdout with the stored `.out` file. Token equality proves
   nothing about binding, so hygiene and fragment rules are
   tested here by what the program prints.
3. Both-reject cases. Rustc must reject each `reject/` fixture and
   its stderr must contain the `// rustc-error:` text in the
   fixture header. Lust must reject these too when it grows macro
   support, with its own L08xx code.

## Run it

```sh
scripts/test-macros.sh            # all cases, rustc mode
scripts/test-macros.sh node_acc   # name filter (substring)
UPDATE=1 scripts/test-macros.sh   # regenerate .out and .expanded
LUST=target/release/lust scripts/test-macros.sh
```

Each step has a timeout: `CASE_TIMEOUT` per compile and
`RUN_TIMEOUT` per run (default 15 seconds each; `TIMEOUT` sets both). A missing
fixture directory fails the run. The runner sets
`LUST_MACRO_TEST_VALUE=hello42` at compile time because
`sem_cfg_builtins` prints `env!` of that name.

In LUST mode the runner also builds and runs pass cases with lust
and checks reject cases with `lust check`. A lust build failure
counts as `unsupported` and does not fail the run.
A lust build with different output, or one that accepts a
both-reject case, counts as `wrong`. That breaks rule zero, but
the count does not fail the run until the W-MACRO gate. Set
`LUST_STRICT=1` to make wrong results and expansion token
differences fail, as in the other corpus runners. The runner also
runs `lust expand` on each supported pass case, canonicalizes its
tokens with the same normalize.py, and diffs them with
`.expanded`.

## ts-rust fixtures

The `macro_rules!` text in each `ts_` fixture is copied exact from
ts-rust. Only the surrounding types are minimal stand-ins. The
fixture header lists the macro names and the source `file:line`
of each definition. Related macros that call each other share one
fixture (the node accessor family is the large case). Run
`python3 tests/macros/check_origin.py` to verify the copies after
an edit. It needs the ts-rust tree (`TS_RUST` env var).

Coverage is a gate, not a note. With no filter, the runner
collects every `// macro:` header and fails if a name from
`ts-rust-names.txt` has no fixture. If ts-rust adds a macro,
rescan, add the name to that file, and add the fixture in the
same change.

Notes on single fixtures:

- `deep/nested/ts_bundled_lib.rs` sits three directories down on
  purpose. The macro includes `concat!("../../../libs/", $name)`
  relative to the invoking file. From there it resolves to
  `tests/macros/libs/`. The file holds both ts-rust definitions
  of the macro. The wasm one is cfg-stripped on this target.
- `ts_child_tests.rs` has a `// mode: test` header. The macros
  generate `#[test]` functions, so the runner also builds it with
  `--test` and runs the tests.
- `ts_payload_codec.rs` exercises the decode half. The encode
  half is `#[cfg(test)]` in the macro output and is not built by
  this bin fixture.
- `ts_node_accessors.rs` is the `by_data!` to `match_data!`
  forwarding path from ARCH 6.2, with the real arm macros.

## Add a fixture

1. Copy the macro text exact. Keep its attributes
   (`#[macro_export]`, cfg) when the source has them.
2. Add the minimum stand-in types so it compiles alone under
   rustc edition 2024. Invoke it the ways the source does and
   print the results.
3. Add the header: `// macro: name` and one `// source:` line
   per definition.
4. Run `UPDATE=1 scripts/test-macros.sh <name>`, read the new
   `.out`, and check the values are the right ones. Do not accept
   output you did not predict.
5. Run the full script with no update flag.
