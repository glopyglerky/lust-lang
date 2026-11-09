# 128-bit integer tests

Gate for the two-word `u128`/`i128` helpers in `src/runtime/core.c`
(ARCH 4.4: tcc has no `__int128`) against Rust 1.99. Run:

```sh
export LUST_TCC_HOME=/path/to/third_party/tcc   # or scripts/build-tcc.sh
tests/int128/test.sh
```

The oracle and the C harness print wrapping arithmetic, signed and
unsigned division and remainder, shifts, comparisons, and decimal
formatting over an edge grid plus 12000 deterministic random pairs
(splitmix64). Outputs must match byte for byte. Checked operations
run one case per process so overflow panics can be checked against
Rust's `checked_*` result and panic text. The first output line
pins size 16 and align 16 for the C types.
