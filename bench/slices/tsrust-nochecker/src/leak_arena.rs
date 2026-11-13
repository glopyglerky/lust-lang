//! Rust-only: the leaked bump arena of a thread, in fixed-size chunks.
//!
//! PERF (rss2): the AST arena (`ast/store.rs`) and the checker arena
//! (`checker/types.rs`) were one `bumpalo::Bump` per thread, with a 1 MiB
//! first chunk. A `Bump` doubles its chunk size each time it grows, so the
//! last chunk of a thread was up to half empty: 1 MiB at the start, then 2,
//! 4 and 8 MiB. Memory that is allocated but never written costs no RSS on
//! 4 KiB pages, but jemalloc gives the heap huge pages (`JEMALLOC_CONF` in
//! `bin/tsgo.rs`), and a huge page is resident as soon as one byte of it is
//! written. That empty tail was 6 MiB of query's peak RSS and 20 MiB of zod's.
//!
//! A `LeakArena` holds one `Bump` of `CHUNK` bytes at a time. Its allocation
//! limit stops it from growing: an allocation that does not fit fails in the
//! `Bump` slow path, and the cold path here leaks a new `Bump` and uses that
//! one. So a thread leaves at most one chunk less than full, and the fast
//! path is the same pointer bump as before.

use std::alloc::Layout;
use std::cell::Cell;
use std::mem::MaybeUninit;

/// The size of each chunk. A thread wastes less than one chunk; a larger
/// chunk costs fewer mallocs (effect: about 800 at 128 KiB).
const CHUNK: usize = 128 << 10;

/// The room to ask of a `Bump` for a malloc block of `CHUNK` bytes: it adds
/// its footer and rounds the block up to whole pages.
const CHUNK_ROOM: usize = CHUNK - 256;

/// A value larger than this does not start a new shared chunk when the
/// current one is full: it gets a chunk of its own, and the current chunk
/// stays in use. So a large value does not throw away the free end of the
/// current chunk.
const OWN_CHUNK_MIN: usize = CHUNK / 4;




#[cfg(test)]
mod tests {
    use super::*;

    /// Values fill a chunk, then go on in a new chunk of the same size; a
    /// large value gets a chunk of its own and the current chunk stays.
    #[test]
    fn chunks_stay_fixed_size() {
        let arena = LeakArena::leak();
        let first = arena.current.get();
        assert!(first.allocated_bytes() <= CHUNK);
        let mut values = Vec::new();
        while std::ptr::eq(arena.current.get(), first) {
            assert!(values.len() <= CHUNK / 64, "the first chunk grew");
            values.push(&*arena.alloc([values.len() as u64; 8]));
        }
        assert!(values.iter().enumerate().all(|(i, v)| v[0] == i as u64));
        let second = arena.current.get();
        assert_eq!(second.allocated_bytes(), first.allocated_bytes());

        let big = arena.alloc_slice_copy(&vec![7u8; CHUNK]);
        assert_eq!(big.len(), CHUNK);
        assert!(std::ptr::eq(arena.current.get(), second));

        let small = arena.alloc_slice_fill_iter(3, [1u32, 2, 3].into_iter());
        assert_eq!(small, &[1, 2, 3]);
    }
}
