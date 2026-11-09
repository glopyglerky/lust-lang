# Adapter: bumpalo 3.20.3

Lust adapter crate (docs/ARCH.md section 3.6) for the `Bump` subset that
ts-rust's `leak_arena.rs` uses. Gate: `scripts/test-adapters.sh bumpalo`.

## Implemented, same results as upstream

- `Bump::new`, `with_capacity`, `set_allocation_limit`,
  `allocation_limit`, `allocated_bytes`, `chunk_capacity`, `alloc`,
  `try_alloc`, `alloc_with`, `try_alloc_with` (the closure runs only
  after the room is found), `alloc_slice_copy`, `try_alloc_slice_copy`,
  `alloc_slice_fill_with`, `try_alloc_slice_fill_with` (a panic in the
  fill closure gives the room back and leaks the items made, as
  upstream's rewind guard does), `Default`; `AllocErr` (Debug, Display, Clone,
  Eq). Features `default`, `std`.
- Values live in leaked boxes and never drop, as in a Bump. Chunk
  sizes, the bump position, limits, and failures are upstream's
  bookkeeping in integers (checked with rustc against the real crate
  over 119,678 random steps).

## Known difference

- An allocation whose alignment is larger than its chunk's (above 16
  unless the chunk was made for it) gets padding that depends on the
  chunk's address in upstream, so `chunk_capacity` after it depends on
  the allocator. The adapter answers as if the chunk start were aligned
  (jemalloc's large allocations, which ts-rust uses).

## Rejected (lust error, never a different answer)

- `Bump<MIN_ALIGN>` other than the default, `reset`, `alloc_str`,
  `alloc_slice_clone`, `iter_allocated_chunks`, the `collections`,
  `boxed`, and `allocator_api` features, and layout queries on `Bump`.
