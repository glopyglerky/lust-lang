# Vec core tests

Gate for the erased Vec and String cores and the `alloc(size, align)`
family in `src/runtime/core.c` (ARCH 3.2, 3.3) against Rust 1.99. Run:

```sh
export LUST_TCC_HOME=/path/to/third_party/tcc   # or scripts/build-tcc.sh
tests/vec/test.sh
```

The oracle prints `capacity()` after each step of push, reserve,
reserve_exact, shrink, with_capacity, and collect sequences for
element sizes 0 (ZST), 1, 2, 4, 8, 16, 1024, and 1025, plus String
growth. The C harness runs the same steps on the erased core.
Outputs must match byte for byte. The harness `alloc` mode checks
alignment 1 to 4 MiB, realloc byte retention, and zeroed alloc.
