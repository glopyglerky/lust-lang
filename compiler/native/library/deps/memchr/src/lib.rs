// Lust adapter for memchr 2.8.2 (docs/ARCH.md section 3.6): byte and
// substring search with the upstream API and the same results. The
// one-needle and forward substring searches call libc through the
// memchr, memrchr, and memmem intrinsics; upstream's SIMD changes speed,
// not answers.
// Module paths follow upstream, so type names print the same
// (`memchr::memchr::Memchr`, `memchr::memmem::Finder`). Items this file
// does not define (`arch`, `memrchr_iter`, `FinderBuilder`, ...) reject
// at the use. No type here has upstream's layout, so `size_of` on one
// rejects (the manifest lists no exact-layout type).

pub use crate::memchr::{Memchr, Memchr2, Memchr3, memchr, memchr_iter, memchr2, memchr2_iter, memchr3, memchr3_iter, memrchr, memrchr2, memrchr3};

mod memchr;
pub mod memmem;
