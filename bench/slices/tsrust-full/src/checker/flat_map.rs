//! PORT: perf. Not in Go. A small open-addressing hash map for hot checker
//! caches (`Relation::results`, `InferenceState::visited`, `InferenceStack::top`).
//!
//! All slots are `(key, value)` pairs in one array, with linear probing and a
//! power-of-two capacity. A probe reads the key from the same cache line as
//! its value; hashbrown reads a control group first and then the slot, which
//! costs one more cache miss on a large table.
//!
//! `K::default()` marks an empty slot. A real key equal to `K::default()` is
//! kept in a side slot. `remove` uses backward-shift deletion, so the map
//! never needs tombstones. There is no iteration, so its layout cannot change
//! any output.
//!
//! The API is the subset of `std::collections::HashMap` that the users need,
//! with the same signatures, so a type alias can switch a map back to
//! hashbrown for an A/B run.

/// A key for `FlatMap`. `flat_hash` must be well mixed in its low bits,
/// because the table index is `flat_hash() & (capacity - 1)`.
pub trait FlatKey: Copy + Eq + Default {
    fn flat_hash(&self) -> u64;
}

/// Growth load: the table grows before more than
/// `slots * MAX_LOAD_NUM / MAX_LOAD_DEN` keys are in it.
const MAX_LOAD_NUM: usize = 1;
const MAX_LOAD_DEN: usize = 2;
/// Slot count of the first allocation.
const MIN_SLOTS: usize = 8;




fn empty_slots<K: FlatKey, V: Copy + Default>(len: usize) -> Box<[(K, V)]> {
    vec![(K::default(), V::default()); len].into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    struct Key(u64);

    impl FlatKey for Key {
        // A weak hash on purpose, so keys collide and probe runs wrap.
        fn flat_hash(&self) -> u64 {
            self.0 % 5
        }
    }

    #[test]
    fn matches_hash_map() {
        let mut flat: FlatMap<Key, u32> = FlatMap::default();
        let mut std_map = std::collections::HashMap::new();
        for n in 0..2000u64 {
            let k = Key((n * 7919) % 613);
            let v = n as u32;
            assert_eq!(flat.insert(k, v), std_map.insert(k, v));
            assert_eq!(flat.len(), std_map.len());
            assert!(flat.len() <= flat.capacity() + 1);
        }
        for n in 0..700u64 {
            assert_eq!(flat.get(&Key(n)), std_map.get(&Key(n)));
        }
        assert!(flat.get(&Key(0)).is_some());
        flat.clear();
        assert_eq!(flat.len(), 0);
        assert_eq!(flat.get(&Key(0)), None);
        assert_eq!(flat.get(&Key(1)), None);
        assert_eq!(flat.insert(Key(1), 3), None);
        assert_eq!(flat.get(&Key(1)), Some(&3));
    }

    /// A key whose even values start probing in the last 5 slots and odd
    /// values in the first 5, so runs wrap the table end and mix with keys
    /// that start at 0.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    struct WrapKey(u64);

    impl FlatKey for WrapKey {
        fn flat_hash(&self) -> u64 {
            if self.0 % 2 == 0 {
                !(self.0 % 5)
            } else {
                self.0 % 5
            }
        }
    }

    #[test]
    fn remove_matches_hash_map() {
        let mut flat: FlatMap<WrapKey, u32> = FlatMap::default();
        let mut std_map = std::collections::HashMap::new();
        assert_eq!(flat.remove(&WrapKey(3)), None);
        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
        for n in 0..20_000u32 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            // Few keys (0 among them), so removes often hit.
            let k = WrapKey((rng >> 8) % 41);
            if rng % 3 == 0 {
                assert_eq!(flat.remove(&k), std_map.remove(&k));
            } else {
                assert_eq!(flat.insert(k, n), std_map.insert(k, n));
            }
            assert_eq!(flat.len(), std_map.len());
            for m in 0..41 {
                assert_eq!(flat.get(&WrapKey(m)), std_map.get(&WrapKey(m)));
            }
        }
        let keys: Vec<WrapKey> = std_map.keys().copied().collect();
        for k in keys {
            assert_eq!(flat.remove(&k), std_map.remove(&k));
        }
        assert_eq!(flat.len(), 0);
        assert!(flat.slots.iter().all(|s| *s == (WrapKey(0), 0)));
    }

    #[test]
    fn with_capacity_and_reserve() {
        assert_eq!(FlatMap::<Key, u32>::with_capacity(0).capacity(), 0);
        for n in 1..300 {
            let cap = FlatMap::<Key, u32>::with_capacity(n).capacity();
            assert!(cap >= n && cap / 2 < n, "n {n} capacity {cap}");
        }
        let mut flat: FlatMap<Key, u32> = FlatMap::default();
        for n in 1..20u64 {
            flat.insert(Key(n), n as u32);
        }
        flat.reserve(1000);
        let cap = flat.capacity();
        assert!(cap >= 1019);
        for n in 20..1020u64 {
            flat.insert(Key(n), n as u32);
        }
        assert_eq!(flat.capacity(), cap);
        for n in 1..1020u64 {
            assert_eq!(flat.get(&Key(n)), Some(&(n as u32)));
        }
    }
}
