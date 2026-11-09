# Layout corpus (ARCH 4.4 gate)

Programs that print `size_of` and `align_of`. `offset_of!` is
used only on `repr(C)` types here, per the A1 gate. ARCH 4.4
rejects `offset_of!` on default-repr types with L0650.

- `pass/*.rs`: rustc accepts and runs. Stdout must equal the
  sibling `.stdout`. Pinned sizes from ARCH 4.4 are also
  written as const asserts in the source, so a wrong layout is
  a compile error, not only an output diff.
- `fail/*.rs`: rustc rejects. The first line is
  `// rustc-error: E####`, as in `tests/traits/fail/`.
- `generate.py`: the ARCH 4.4 shape generator. It enumerates
  the supported-shape grammar with stated bounds (see the
  constants at its top), emits `pass/gen_shapes.rs`, and with
  `--write` records `pass/gen_shapes.stdout` from rustc.
  Run without `--write` to check that the committed files are
  the generator's current output.

Runner: `scripts/test-adapters.sh`. In rustc mode it also
re-runs the generator check. Sizes assume a 64-bit target,
like the ts-rust asserts that pin them.
