// Lust adapter for xxhash-rust 0.8.15 (docs/ARCH.md section 3.6): the
// `xxh3` module only (feature `xxh3`). The hashing is C in the runtime
// (src/runtime/deps.c, one intrinsic each for 64 and 128 bits); this
// crate is the thin source API. Other features (xxh32, xxh64, the const
// forms, std) are not implemented: enabling one is L0508.

#[cfg(feature = "xxh3")]
pub mod xxh3;
