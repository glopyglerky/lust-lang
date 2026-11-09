// Lust adapter for rustc-hash 2.1.2 (docs/ARCH.md section 3.6): the Fx
// hasher and the map and set aliases over std's HashMap and HashSet.
// Upstream is already only aliases plus the hasher, so this is upstream's
// stable (non-nightly) code with the byte reads written out. Not here:
// features `nightly` and `rand` (FxRandomState, FxHashMapRand), L0508.

mod seeded_state;

use core::hash::{BuildHasher, Hasher};
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

/// A hash map with the Fx hasher.
#[cfg(feature = "std")]
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

/// A hash set with the Fx hasher.
#[cfg(feature = "std")]
pub type FxHashSet<V> = HashSet<V, FxBuildHasher>;

pub use seeded_state::FxSeededState;
#[cfg(feature = "std")]
pub use seeded_state::{FxHashMapSeed, FxHashSetSeed};

/// The Fx hasher: a polynomial hash with a rotate at `finish`.
#[derive(Clone)]
pub struct FxHasher {
    hash: usize,
}

/// Upstream's `K`, renamed: lust resolves the type param `K` of the
/// re-exported `FxHashMapSeed<K, V>` to a const of that name here
/// (repro in the W-DEPS report). Private, so the name is not visible.
const MULTIPLIER: usize = 0xf1357aea2e62a9c5;

impl FxHasher {
    /// A hasher that starts from `seed`.
    pub const fn with_seed(seed: usize) -> FxHasher {
        FxHasher { hash: seed }
    }

    /// A hasher that starts from 0.
    pub const fn default() -> FxHasher {
        FxHasher { hash: 0 }
    }

    fn add_to_hash(&mut self, i: usize) {
        self.hash = self.hash.wrapping_add(i).wrapping_mul(MULTIPLIER);
    }
}

impl Default for FxHasher {
    fn default() -> FxHasher {
        FxHasher { hash: 0 }
    }
}

impl Hasher for FxHasher {
    /// Upstream calls `self.write_u64`; on 64-bit targets that is this.
    fn write(&mut self, bytes: &[u8]) {
        self.add_to_hash(hash_bytes(bytes) as usize);
    }

    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as usize);
    }

    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as usize);
    }

    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as usize);
    }

    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as usize);
    }

    fn write_u128(&mut self, i: u128) {
        self.add_to_hash(i as usize);
        self.add_to_hash((i >> 64) as usize);
    }

    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i);
    }

    fn finish(&self) -> u64 {
        self.hash.rotate_left(26) as u64
    }
}

// Digits of pi.
const SEED1: u64 = 0x243f6a8885a308d3;
const SEED2: u64 = 0x13198a2e03707344;
const PREVENT_TRIVIAL_ZERO_COLLAPSE: u64 = 0xa4093822299f31d0;

/// The xor of the two halves of the full 128-bit product.
fn multiply_mix(x: u64, y: u64) -> u64 {
    let full = (x as u128).wrapping_mul(y as u128);
    (full as u64) ^ ((full >> 64) as u64)
}

// Little-endian reads; `hash_bytes` calls them only in bounds. One copy of 8 (or 4)
// bytes, which a release build's C compiler makes a single load.
fn read_u64(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

fn read_u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

/// Compresses a byte string to one u64 (little-endian reads, as upstream).
fn hash_bytes(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut s0 = SEED1;
    let mut s1 = SEED2;
    if len <= 16 {
        if len >= 8 {
            s0 ^= read_u64(bytes, 0);
            s1 ^= read_u64(bytes, len - 8);
        } else if len >= 4 {
            s0 ^= read_u32(bytes, 0) as u64;
            s1 ^= read_u32(bytes, len - 4) as u64;
        } else if len > 0 {
            let lo = bytes[0];
            let mid = bytes[len / 2];
            let hi = bytes[len - 1];
            s0 ^= lo as u64;
            s1 ^= ((hi as u64) << 8) | mid as u64;
        }
    } else {
        // Whole 16-byte chunks of bytes[..len - 1], then the last 16 bytes.
        let mut off = 0;
        while off + 16 <= len - 1 {
            let x = read_u64(bytes, off);
            let y = read_u64(bytes, off + 8);
            let t = multiply_mix(s0 ^ x, PREVENT_TRIVIAL_ZERO_COLLAPSE ^ y);
            s0 = s1;
            s1 = t;
            off += 16;
        }
        s0 ^= read_u64(bytes, len - 16);
        s1 ^= read_u64(bytes, len - 8);
    }
    multiply_mix(s0, s1) ^ (len as u64)
}

/// Builds `FxHasher`s that start from 0.
#[derive(Copy, Clone, Default)]
pub struct FxBuildHasher;

impl BuildHasher for FxBuildHasher {
    type Hasher = FxHasher;
    fn build_hasher(&self) -> FxHasher {
        FxHasher::default()
    }
}
