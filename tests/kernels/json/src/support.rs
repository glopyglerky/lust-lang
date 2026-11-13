//! Local stand-ins for the external crates the ts-rust kernel sources use.
//!
//! Each kernel in `tests/kernels/` is a standalone crate with no
//! dependencies, so this file is copied into each kernel that needs it.
//! Keep the copies in sync. Replacements, with the ts-rust pinned version:
//!
//! - `memchr` 2.8: naive byte search, same API and results. Speed does
//!   not matter for a fixture; positions do, and they match by definition.
//! - `rustc-hash` 2.1 (`FxHashMap`, `FxHashSet`, `FxBuildHasher`): the
//!   FxHash algorithm (rotate-left 5, xor, multiply by the Fx seed) over
//!   `std::collections` maps. Kernel drivers never print map iteration
//!   order, so only membership behavior is load-bearing here.
//! - `indexmap` 2.14 (`IndexMap`, `IndexSet`): insertion-ordered map over
//!   a `Vec` with linear search. Iteration order matches the real crate
//!   (insertion order), which the JSON kernel output does depend on.

/// Stand-in for the ts-rust `unported!` macro (`gopanic.rs`): it panics
/// with the same text. The recording side effect is dropped; no kernel
/// driver reaches an unported path.
#[macro_export]
macro_rules! unported {
    ($go_name:expr) => {{
        panic!("unported Go code: {}", $go_name)
    }};
}

/// Stand-in for the `memchr` crate: first/next byte search in a slice.
pub mod memchr {
    /// `memchr::memchr`: first index of `needle` in `haystack`.
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    /// `memchr::memchr2`: first index of either needle.
    pub fn memchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == n1 || b == n2)
    }

    /// `memchr::memchr3`: first index of any of the three needles.
    pub fn memchr3(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == n1 || b == n2 || b == n3)
    }

    /// `memchr::memrchr`: last index of `needle`.
    pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().rposition(|&b| b == needle)
    }

    /// `memchr::memrchr2`: last index of either needle.
    pub fn memrchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().rposition(|&b| b == n1 || b == n2)
    }

    /// Collect all match positions into an owning iterator.
    fn all(haystack: &[u8], pred: impl Fn(u8) -> bool) -> std::vec::IntoIter<usize> {
        let mut out = Vec::new();
        for (i, &b) in haystack.iter().enumerate() {
            if pred(b) {
                out.push(i);
            }
        }
        out.into_iter()
    }

    /// `memchr::memchr_iter`: every index of `needle`.
    pub fn memchr_iter(needle: u8, haystack: &[u8]) -> std::vec::IntoIter<usize> {
        all(haystack, |b| b == needle)
    }

    /// `memchr::memchr2_iter`: every index of either needle.
    pub fn memchr2_iter(n1: u8, n2: u8, haystack: &[u8]) -> std::vec::IntoIter<usize> {
        all(haystack, |b| b == n1 || b == n2)
    }

    /// `memchr::memchr3_iter`: every index of any of the three needles.
    pub fn memchr3_iter(n1: u8, n2: u8, n3: u8, haystack: &[u8]) -> std::vec::IntoIter<usize> {
        all(haystack, |b| b == n1 || b == n2 || b == n3)
    }

    /// Stand-in for `memchr::memmem` (substring search).
    pub mod memmem {
        /// `memchr::memmem::find`: first index of `needle` in `haystack`.
        pub fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            if needle.is_empty() {
                return Some(0);
            }
            haystack.windows(needle.len()).position(|w| w == needle)
        }

        /// `memchr::memmem::Finder`: a reusable substring searcher.
        pub struct Finder<'n> {
            needle: &'n [u8],
        }

        impl<'n> Finder<'n> {
            pub fn new<B: AsRef<[u8]> + ?Sized>(needle: &'n B) -> Finder<'n> {
                Finder {
                    needle: needle.as_ref(),
                }
            }

            pub fn find(&self, haystack: &[u8]) -> Option<usize> {
                find(haystack, self.needle)
            }
        }
    }
}

/// Stand-in for the `rustc-hash` crate (FxHash over std maps).
pub mod rustc_hash {
    use std::hash::{BuildHasher, Hasher};

    /// The FxHash seed constant (rustc-hash 2.x).
    const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

    /// `rustc_hash::FxHasher`: rotate, xor, multiply per chunk.
    #[derive(Default)]
    pub struct FxHasher {
        hash: u64,
    }

    impl FxHasher {
        fn add(&mut self, word: u64) {
            self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
        }
    }

    impl Hasher for FxHasher {
        fn finish(&self) -> u64 {
            self.hash
        }

        fn write(&mut self, mut bytes: &[u8]) {
            while bytes.len() >= 8 {
                self.add(u64::from_le_bytes(bytes[..8].try_into().unwrap()));
                bytes = &bytes[8..];
            }
            if bytes.len() >= 4 {
                self.add(u64::from(u32::from_le_bytes(bytes[..4].try_into().unwrap())));
                bytes = &bytes[4..];
            }
            if bytes.len() >= 2 {
                self.add(u64::from(u16::from_le_bytes(bytes[..2].try_into().unwrap())));
                bytes = &bytes[2..];
            }
            if !bytes.is_empty() {
                self.add(u64::from(bytes[0]));
            }
        }

        fn write_u8(&mut self, n: u8) {
            self.add(u64::from(n));
        }
        fn write_u16(&mut self, n: u16) {
            self.add(u64::from(n));
        }
        fn write_u32(&mut self, n: u32) {
            self.add(u64::from(n));
        }
        fn write_u64(&mut self, n: u64) {
            self.add(n);
        }
        fn write_usize(&mut self, n: usize) {
            self.add(n as u64);
        }
        fn write_i8(&mut self, n: i8) {
            self.add(n as u64);
        }
        fn write_i16(&mut self, n: i16) {
            self.add(n as u64);
        }
        fn write_i32(&mut self, n: i32) {
            self.add(n as u64);
        }
        fn write_i64(&mut self, n: i64) {
            self.add(n as u64);
        }
        fn write_isize(&mut self, n: isize) {
            self.add(n as u64);
        }
    }

    /// `rustc_hash::FxBuildHasher`: a unit struct in the real crate,
    /// passed by value to with_hasher.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct FxBuildHasher;

    impl BuildHasher for FxBuildHasher {
        type Hasher = FxHasher;

        fn build_hasher(&self) -> FxHasher {
            FxHasher::default()
        }
    }

    /// `rustc_hash::FxHashMap`.
    pub type FxHashMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;

    /// `rustc_hash::FxHashSet`.
    pub type FxHashSet<T> = std::collections::HashSet<T, FxBuildHasher>;
}

/// Stand-in for the `indexmap` crate: insertion-ordered map and set.
pub mod indexmap {
    use std::borrow::Borrow;
    use std::hash::Hash;

    /// `indexmap::IndexMap` with the subset of the API the kernels use.
    /// Iteration is in insertion order, as in the real crate. The `S`
    /// hasher parameter exists for API shape only; search is linear.
    #[derive(Clone)]
    pub struct IndexMap<K, V, S = crate::support::rustc_hash::FxBuildHasher> {
        entries: Vec<(K, V)>,
        hasher: std::marker::PhantomData<S>,
    }

    /// Debug prints as a map, as the real crate does.
    impl<K: std::fmt::Debug, V: std::fmt::Debug, S> std::fmt::Debug for IndexMap<K, V, S> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map().entries(self.entries.iter().map(|(k, v)| (k, v))).finish()
        }
    }

    impl<K, V, S> Default for IndexMap<K, V, S> {
        fn default() -> Self {
            IndexMap {
                entries: Vec::new(),
                hasher: std::marker::PhantomData,
            }
        }
    }

    /// Map equality in indexmap is order-independent: same keys with
    /// equal values, in any order.
    impl<K: Eq + Hash, V: PartialEq, S, S2> PartialEq<IndexMap<K, V, S2>> for IndexMap<K, V, S> {
        fn eq(&self, other: &IndexMap<K, V, S2>) -> bool {
            self.entries.len() == other.entries.len()
                && self
                    .entries
                    .iter()
                    .all(|(k, v)| other.entries.iter().any(|(ok, ov)| ok == k && ov == v))
        }
    }

    impl<K: Eq + Hash, V: Eq, S> Eq for IndexMap<K, V, S> {}

    impl<K, V, S> IndexMap<K, V, S> {
        pub fn new() -> Self {
            IndexMap::default()
        }

        pub fn len(&self) -> usize {
            self.entries.len()
        }

        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }

        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.entries.iter().map(|(k, v)| (k, v))
        }

        pub fn keys(&self) -> impl Iterator<Item = &K> {
            self.entries.iter().map(|(k, _)| k)
        }

        pub fn values(&self) -> impl Iterator<Item = &V> {
            self.entries.iter().map(|(_, v)| v)
        }

        pub fn clear(&mut self) {
            self.entries.clear();
        }
    }

    impl<K: Eq + Hash, V, S> IndexMap<K, V, S> {

        /// Insert; returns the old value. A repeated key keeps its
        /// first position, as in the real crate.
        pub fn insert(&mut self, key: K, value: V) -> Option<V> {
            for e in &mut self.entries {
                if e.0 == key {
                    return Some(std::mem::replace(&mut e.1, value));
                }
            }
            self.entries.push((key, value));
            None
        }

        pub fn get<Q>(&self, key: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
            Q: Eq + Hash + ?Sized,
        {
            self.entries.iter().find(|e| e.0.borrow() == key).map(|e| &e.1)
        }

        pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
        where
            K: Borrow<Q>,
            Q: Eq + Hash + ?Sized,
        {
            self.entries.iter_mut().find(|e| e.0.borrow() == key).map(|e| &mut e.1)
        }

        pub fn contains_key<Q>(&self, key: &Q) -> bool
        where
            K: Borrow<Q>,
            Q: Eq + Hash + ?Sized,
        {
            self.get(key).is_some()
        }

        /// Remove and return the value; later entries shift down
        /// (`shift_remove` semantics, order preserved).
        pub fn shift_remove<Q>(&mut self, key: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: Eq + Hash + ?Sized,
        {
            let pos = self.entries.iter().position(|e| e.0.borrow() == key)?;
            Some(self.entries.remove(pos).1)
        }

        pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: Eq + Hash + ?Sized,
        {
            self.shift_remove(key)
        }

        /// Entry API subset: mutable reference to the value for `key`,
        /// inserting `default` first when absent.
        pub fn entry_or_insert(&mut self, key: K, default: V) -> &mut V
        where
            K: Clone,
        {
            if let Some(pos) = self.entries.iter().position(|e| e.0 == key) {
                return &mut self.entries[pos].1;
            }
            self.entries.push((key, default));
            let last = self.entries.len() - 1;
            &mut self.entries[last].1
        }
    }

    impl<K: Eq + Hash, V, S> FromIterator<(K, V)> for IndexMap<K, V, S> {
        fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
            let mut map = IndexMap::new();
            for (k, v) in iter {
                map.insert(k, v);
            }
            map
        }
    }

    impl<'a, K, V, S> IntoIterator for &'a IndexMap<K, V, S> {
        type Item = (&'a K, &'a V);
        type IntoIter = std::iter::Map<
            std::slice::Iter<'a, (K, V)>,
            fn(&'a (K, V)) -> (&'a K, &'a V),
        >;
        fn into_iter(self) -> Self::IntoIter {
            self.entries.iter().map(|(k, v)| (k, v))
        }
    }

    impl<K, V, S> IntoIterator for IndexMap<K, V, S> {
        type Item = (K, V);
        type IntoIter = std::vec::IntoIter<(K, V)>;
        fn into_iter(self) -> Self::IntoIter {
            self.entries.into_iter()
        }
    }

    /// `indexmap::IndexSet` with the subset of the API the kernels use.
    /// The `S` hasher parameter exists for API shape only.
    #[derive(Clone)]
    pub struct IndexSet<T, S = crate::support::rustc_hash::FxBuildHasher> {
        entries: Vec<T>,
        hasher: std::marker::PhantomData<S>,
    }

    impl<T, S> Default for IndexSet<T, S> {
        fn default() -> Self {
            IndexSet {
                entries: Vec::new(),
                hasher: std::marker::PhantomData,
            }
        }
    }

    /// Debug prints as a set, as the real crate does.
    impl<T: std::fmt::Debug, S> std::fmt::Debug for IndexSet<T, S> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_set().entries(self.entries.iter()).finish()
        }
    }

    /// Set equality in indexmap is order-independent.
    impl<T: Eq + Hash, S, S2> PartialEq<IndexSet<T, S2>> for IndexSet<T, S> {
        fn eq(&self, other: &IndexSet<T, S2>) -> bool {
            self.entries.len() == other.entries.len()
                && self.entries.iter().all(|v| other.entries.contains(v))
        }
    }

    impl<T: Eq + Hash, S> Eq for IndexSet<T, S> {}

    impl<T, S> IndexSet<T, S> {
        pub fn new() -> Self {
            IndexSet::default()
        }

        pub fn len(&self) -> usize {
            self.entries.len()
        }

        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }

        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.entries.iter()
        }

        pub fn clear(&mut self) {
            self.entries.clear();
        }
    }

    impl<T: Eq + Hash, S> IndexSet<T, S> {

        pub fn insert(&mut self, value: T) -> bool {
            if self.entries.contains(&value) {
                return false;
            }
            self.entries.push(value);
            true
        }

        pub fn contains<Q>(&self, value: &Q) -> bool
        where
            T: Borrow<Q>,
            Q: Eq + Hash + ?Sized,
        {
            self.entries.iter().any(|e| e.borrow() == value)
        }
    }

    impl<T: Eq + Hash, S> FromIterator<T> for IndexSet<T, S> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut set = IndexSet::new();
            for v in iter {
                set.insert(v);
            }
            set
        }
    }
}
