# Adapter: xxhash-rust 0.8.15

Lust adapter crate (docs/ARCH.md section 3.6). Dev builds compile it in
place of the registry package xxhash-rust 0.8.15 (feature `xxh3`);
release builds use the real crate through Cargo. Gate:
`scripts/test-adapters.sh xxhash-rust` (the real crate is the oracle).

## Implemented, same results as upstream

- `xxh3::{xxh3_64, xxh3_64_with_seed, xxh3_128, xxh3_128_with_seed}`:
  one intrinsic each for 64 and 128 bits, over a scalar C port of
  upstream's `src/xxh3.rs` (src/runtime/deps.c).
- `xxh3::Xxh3`: `new`, `with_seed`, `reset`, `update`, `digest`,
  `digest128`, `Clone`, `Default`. It keeps its input and hashes all of
  it at `digest`, which XXH3 defines to equal the streaming digest.
- Feature: `xxh3`.

## Rejected (lust error, never a different answer)

- The `_with_secret` forms, `Xxh3::with_secret`, `Xxh3Default`,
  `Xxh3Builder`, `Xxh3DefaultBuilder`, and the `Hasher` and `io::Write`
  impls (L0506 or L0104 at the use).
- Features `xxh32`, `xxh64`, `const_xxh3`, `const_xxh32`,
  `const_xxh64`, `std` (L0508).
- `size_of` and other layout queries on `Xxh3` (L0652).
