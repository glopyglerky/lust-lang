# Adapter: tikv-jemallocator 0.7.0

Lust adapter crate (docs/ARCH.md section 3.6) for the `Jemalloc`
type that ts-rust's five `bin/*.rs` roots name in their
`#[global_allocator]` statics. Gate:
`scripts/test-adapters.sh tikv-jemallocator`.

## Implemented, same results as upstream

- `Jemalloc`: a unit struct with upstream's `Copy`, `Clone`,
  `Default`, and `Debug` derives. Features `default` and
  `background_threads_runtime_support` (they change only the
  jemalloc build upstream, not the API).
- A `#[global_allocator]` static of this type compiles. The lust
  runtime keeps its own allocator: allocation strategy does not
  change a program's output or exit code, so dev and release
  builds (release uses the real crate through Cargo) agree on
  every observable result except speed and memory use.

## Rejected (lust error, never a different answer)

- The `GlobalAlloc` methods and `usable_size` (all `unsafe`),
  the `alloc_trait`, `profiling`, `stats`, and other
  build-tuning features, and a second `#[global_allocator]`
  static or one of any other type.
