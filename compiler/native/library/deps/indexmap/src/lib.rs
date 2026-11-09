// Lust adapter for indexmap 2.14.0 (docs/ARCH.md section 3.6): IndexMap
// and IndexSet in source. The hash index is hashbrown 0.17.1's table
// logic for usize slots (indices.rs) and the core is upstream's
// src/inner.rs, so insertion order, capacity, and the Hash and Eq calls
// a key sees match upstream for the same hasher. Not here (they reject
// at the use): Equivalent (lookups take `K: Borrow<Q>` as std maps do),
// most of map::Slice (as_slice and its read-only core are here), drain, splice, raw entries, sorted inserts, the rayon, serde,
// and borsh features, and the indexmap!/indexset! macros.

mod indices;
mod inner;
pub mod map;
pub mod set;

pub use crate::map::IndexMap;
pub use crate::set::IndexSet;
