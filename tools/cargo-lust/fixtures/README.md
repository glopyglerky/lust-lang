# Compiler qualification fixtures

`full-rust` is a dependency-free Cargo workspace with three real crates. It
exercises an external procedural derive, a build script's generated Rust and
environment variables, async polling, generic associated types, const generics,
unsafe pointer access, a C calling convention, cross-crate calls, and a doctest.
The binary catches a panic and checks that a stack destructor ran. This catches
an accidental change from unwinding to aborting.

Expected output with no application arguments:

```text
derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1
```

The `BENCH_PROBE` line in `full-rust/core/src/lib.rs` is an edit point for paired
benchmark copies. The binary casts its result to `u64`, so changing its return
type from `u32` to `u64` preserves the printed value. Benchmark code must edit a
private copy of the fixture, never the repository's sources.

`invalid-borrow` must fail with rustc error `E0502`. The compatibility engines
must retain Cargo's failure status.

`subset/supported.rs` is a small Lust program with output `42`.
`subset/unsupported.rs` is valid Rust that Lust rejects with `L0405`. The explicit
Lust engine must keep that rejection; it must not silently run another compiler.

These tests establish agreement on this corpus. They do not prove arbitrary
Rust compatibility, arbitrary backend interoperability, or a compilation speedup.
