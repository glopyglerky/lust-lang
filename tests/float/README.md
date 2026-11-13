# Float runtime tests

Tests for `src/runtime/float.c`: byte-exact Rust 1.99 float formatting,
parsing, casts, and ryu-js (ECMAScript) number formatting.

- `harness.c` compiles the real runtime with a C compiler and answers
  case streams. `harness edges` runs directed assertions with no
  oracle. `harness bench` prints ns per call.
- `oracle.rs` is the rustc reference for the same case streams.
- `oracle_js.rs` is the ryu_js 1.0.2 crate reference (JS numbers).
- `gen.c` makes biased random cases: bit patterns around subnormals,
  binade edges, powers of ten, exact-tie values; decimal strings
  around halfway points and overflow edges; grammar fuzz.
- `fuzz.sh` diffs harness against oracle output. Default run is
  20,000 cases per function for CI. The full gate is:

  ```sh
  LUST_TCC_HOME=third_party/tcc tests/float/fuzz.sh --full --bench
  ```

  `--full` is 10,000,000 cases per function (1,000,000 format-spec
  combos). Set `COUNT` or `SEED` to override. The harness is built
  with the pinned tcc, the same compiler lust uses, plus a gcc build
  checked on a slice to prove the answers do not depend on the C
  compiler.
- `gen_ryu_tables.py` regenerates `src/runtime/ryu_tables.h` from the
  ryu-js crate in the local cargo registry.

Two measured facts the implementation depends on, both re-checked by
the gate: Rust's shortest printing rounds exact ties up (value ends
in decimal ...5 with zeros after), while ECMAScript prints the even
candidate; and Rust's `{:.N}` rounds the exact value to nearest,
ties to even.
