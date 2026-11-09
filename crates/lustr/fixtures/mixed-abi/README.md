# Mixed LLVM / Cranelift ABI contract

This path-only workspace checks calls between separately compiled Rust crates.
`mixed-abi-contract` defines shared types; `mixed-abi-provider` implements the
provider; `mixed-abi-app` implements callbacks and runs assertions. Boundary
functions use `#[inline(never)]`. Both backends use the same pinned rustc
frontend and its ordinary LLVM-built standard library.

The integration test builds a fresh ordinary Cargo baseline, then two fresh
mixed configurations:

| Configuration | Contract | Provider | App and callbacks |
| --- | --- | --- | --- |
| LLVM baseline | LLVM | LLVM | LLVM |
| Cranelift consumer | LLVM | LLVM | Cranelift |
| Cranelift provider | LLVM | Cranelift | LLVM |

Callbacks cross back into the caller, so each mixed configuration executes
calls in both directions. The reverse configuration also tests each crate's
implementation with the other code generator. The gate reads actual `cargo -vv`
compiler commands, checks every crate's selected backend, rejects absent fresh
compilations and requires the pinned rustc commit. A failed backend load,
compiler invocation, link, execution or assertion fails the gate. No retry
through another backend occurs.

The executable and Cargo test harness verify:

- Rust-layout aggregates passed and returned by value, including a `u128`,
  fixed arrays, `String`, `Vec` and an optional nonzero integer;
- generic monomorphizations of distinct integer types, const generics, and a
  function-pointer callback returning a generic aggregate;
- trait-object method calls and dynamic destructors in both directions;
- present and absent nonzero, owned-pointer and borrowed-pointer niches, plus
  enum variants containing owned allocations;
- allocations created by one crate and grown or freed by the other;
- panic recovery after unwinding through the other backend's stack frames,
  with exact destructor order for both panic origins;
- atomic compare/exchange plus concurrent updates through both crates;
- distinct thread-local values and thread-exit destructors in both crates;
- a `repr(C)` aggregate passed and returned through an `extern "C"` callback.
  This last check uses Rust-defined C calling conventions, without a separate
  C compiler or allocator.

Run the fixture without a network or registry dependencies:

```sh
cargo run --manifest-path crates/lustr/fixtures/mixed-abi/Cargo.toml --offline --locked
cargo test --manifest-path crates/lustr/Cargo.toml --offline --locked --test mixed_abi
```

After building the pinned unwind-capable backend with the repository setup
helper, run the actual mixed gate:

```sh
source crates/lustr/target/complement/env.sh
cargo test --manifest-path crates/lustr/Cargo.toml --offline --locked   --test mixed_abi actual_cranelift -- --ignored --nocapture
```

`LUST_TEST_ABI_EVIDENCE=/absolute/directory` retains raw stdout, stderr and exit
status for each compilation, direct execution, test command and compiler identity
probe. Each run gets a unique directory. A `passed.txt` receipt is written only
after every assertion succeeds; failed runs retain their failing command output.

`LUST_TEST_CARGO` may select an explicit Cargo binary. The backend comes from
`LUST_CRANELIFT_BACKEND`; it is required, never auto-installed. Ignoring the
optional gate is not backend qualification. This is a bounded regression
contract for the pinned Linux x86_64 LLVM/Cranelift pair, not native Lust rlib
interoperability or proof of compatibility for every Rust program or platform.
