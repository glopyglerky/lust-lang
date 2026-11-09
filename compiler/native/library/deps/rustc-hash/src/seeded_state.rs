// `FxSeededState`: builds Fx hashers that start from a fixed seed.

use crate::FxHasher;

/// A hash map whose Fx hashers start from a seed.
#[cfg(feature = "std")]
pub type FxHashMapSeed<K, V> = std::collections::HashMap<K, V, FxSeededState>;

/// A hash set whose Fx hashers start from a seed.
#[cfg(feature = "std")]
pub type FxHashSetSeed<V> = std::collections::HashSet<V, FxSeededState>;

/// Builds `FxHasher`s that start from `seed`.
#[derive(Clone)]
pub struct FxSeededState {
    seed: usize,
}

impl FxSeededState {
    pub const fn with_seed(seed: usize) -> FxSeededState {
        Self { seed }
    }
}

impl core::hash::BuildHasher for FxSeededState {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FxHasher::with_seed(self.seed)
    }
}
