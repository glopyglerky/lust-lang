// Lust adapter for tikv-jemallocator 0.7.0 (docs/ARCH.md section
// 3.6): the `Jemalloc` type only. A `#[global_allocator]` static
// of this type is accepted and the lust runtime keeps its own
// allocator (allocation strategy does not change program output);
// collect enforces that rule. Not here: the `GlobalAlloc` impl
// surface (`alloc`, `dealloc`, and kin, all `unsafe`), the
// `usable_size` fn, and the `alloc_trait` feature. Code that
// names them rejects in lust, never answers differently.

/// Handle to the jemalloc allocator. A unit struct, as upstream.
#[derive(Copy, Clone, Default, Debug)]
pub struct Jemalloc;
