// `indexmap::set`: IndexSet, an IndexMap<T, ()> as upstream builds it,
// so every operation reaches the index the same way.

use crate::map::{self, IndexMap};
use core::hash::{BuildHasher, Hash};
#[cfg(feature = "std")]
use std::collections::hash_map::RandomState;

/// A hash set that keeps insertion order (`S` defaults to `RandomState`
/// only with `std`, as upstream).
#[cfg(feature = "std")]
pub struct IndexSet<T, S = RandomState> {
    pub(crate) map: IndexMap<T, (), S>,
}

/// A hash set that keeps insertion order (no `std`: no default hasher).
#[cfg(not(feature = "std"))]
pub struct IndexSet<T, S> {
    pub(crate) map: IndexMap<T, (), S>,
}

#[cfg(feature = "std")]
impl<T> IndexSet<T> {
    pub fn new() -> Self {
        IndexSet { map: IndexMap::new() }
    }

    pub fn with_capacity(n: usize) -> Self {
        IndexSet { map: IndexMap::with_capacity(n) }
    }
}

impl<T, S> IndexSet<T, S> {
    pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
        IndexSet { map: IndexMap::with_capacity_and_hasher(n, hash_builder) }
    }

    pub fn with_hasher(hash_builder: S) -> Self {
        IndexSet { map: IndexMap::with_hasher(hash_builder) }
    }

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter { inner: self.map.keys() }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn truncate(&mut self, len: usize) {
        self.map.truncate(len);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.map.pop() {
            Some((x, ())) => Some(x),
            None => None,
        }
    }

    pub fn retain<F: FnMut(&T) -> bool>(&mut self, mut keep: F) {
        self.map.retain(|x, _| keep(x));
    }

    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.map.sort_keys();
    }

    pub fn sort_by<F: FnMut(&T, &T) -> core::cmp::Ordering>(&mut self, mut cmp: F) {
        self.map.sort_by(|a, _, b, _| cmp(a, b));
    }

    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.map.sort_unstable_keys();
    }

    pub fn reverse(&mut self) {
        self.map.reverse();
    }

    pub fn get_index(&self, index: usize) -> Option<&T> {
        match self.map.get_index(index) {
            Some((x, _)) => Some(x),
            None => None,
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.get_index(0)
    }

    pub fn last(&self) -> Option<&T> {
        match self.map.last() {
            Some((x, _)) => Some(x),
            None => None,
        }
    }

    pub fn swap_remove_index(&mut self, index: usize) -> Option<T> {
        match self.map.swap_remove_index(index) {
            Some((x, ())) => Some(x),
            None => None,
        }
    }

    pub fn shift_remove_index(&mut self, index: usize) -> Option<T> {
        match self.map.shift_remove_index(index) {
            Some((x, ())) => Some(x),
            None => None,
        }
    }
}

impl<T: Hash + Eq, S: BuildHasher> IndexSet<T, S> {
    /// True when `value` was new (upstream: `map.insert(value, ())`).
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    pub fn insert_full(&mut self, value: T) -> (usize, bool) {
        let (index, existing) = self.map.insert_full(value, ());
        (index, existing.is_none())
    }

    pub fn contains<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> bool
    where
        T: core::borrow::Borrow<Q>,
    {
        self.map.contains_key(value)
    }

    pub fn get<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> Option<&T>
    where
        T: core::borrow::Borrow<Q>,
    {
        match self.map.get_key_value(value) {
            Some((x, _)) => Some(x),
            None => None,
        }
    }

    pub fn get_full<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> Option<(usize, &T)>
    where
        T: core::borrow::Borrow<Q>,
    {
        match self.map.get_full(value) {
            Some((i, x, _)) => Some((i, x)),
            None => None,
        }
    }

    pub fn get_index_of<Q: ?Sized + Hash + Eq>(&self, value: &Q) -> Option<usize>
    where
        T: core::borrow::Borrow<Q>,
    {
        self.map.get_index_of(value)
    }

    pub fn swap_remove<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> bool
    where
        T: core::borrow::Borrow<Q>,
    {
        self.map.swap_remove(value).is_some()
    }

    pub fn shift_remove<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> bool
    where
        T: core::borrow::Borrow<Q>,
    {
        self.map.shift_remove(value).is_some()
    }

    /// Upstream deprecates `remove`; it is `swap_remove`.
    pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> bool
    where
        T: core::borrow::Borrow<Q>,
    {
        self.swap_remove(value)
    }

    pub fn swap_take<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> Option<T>
    where
        T: core::borrow::Borrow<Q>,
    {
        match self.map.swap_remove_entry(value) {
            Some((x, ())) => Some(x),
            None => None,
        }
    }

    pub fn shift_take<Q: ?Sized + Hash + Eq>(&mut self, value: &Q) -> Option<T>
    where
        T: core::borrow::Borrow<Q>,
    {
        match self.map.shift_remove_entry(value) {
            Some((x, ())) => Some(x),
            None => None,
        }
    }

    /// True when every value of `self` is in `other`.
    pub fn is_subset<S2: BuildHasher>(&self, other: &IndexSet<T, S2>) -> bool {
        if self.len() > other.len() {
            return false;
        }
        for value in self.iter() {
            if !other.contains(value) {
                return false;
            }
        }
        true
    }
}

/// The values in order.
pub struct Iter<'a, T> {
    inner: map::Keys<'a, T, ()>,
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter { inner: self.inner.clone() }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.inner.next_back()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T, S> IntoIterator for &'a IndexSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

/// The values in order, by value.
pub struct IntoIter<T> {
    inner: map::IntoIter<T, ()>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.inner.next() {
            Some((x, ())) => Some(x),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        match self.inner.next_back() {
            Some((x, ())) => Some(x),
            None => None,
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T, S> IntoIterator for IndexSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { inner: self.map.into_iter() }
    }
}

impl<T: Clone, S: Clone> Clone for IndexSet<T, S> {
    fn clone(&self) -> Self {
        IndexSet { map: self.map.clone() }
    }
}

impl<T, S: Default> Default for IndexSet<T, S> {
    fn default() -> Self {
        IndexSet { map: IndexMap::default() }
    }
}

impl<T: core::fmt::Debug, S> core::fmt::Debug for IndexSet<T, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Upstream's `entries(self.iter())`, one entry at a time.
        let mut set = f.debug_set();
        let entries = &self.map.core.entries;
        let mut i = 0;
        while i < entries.len() {
            set.entry(&entries[i].key);
            i += 1;
        }
        set.finish()
    }
}

impl<T: Hash + Eq, S1: BuildHasher, S2: BuildHasher> PartialEq<IndexSet<T, S2>> for IndexSet<T, S1> {
    fn eq(&self, other: &IndexSet<T, S2>) -> bool {
        self.len() == other.len() && self.is_subset(other)
    }
}

impl<T: Hash + Eq, S: BuildHasher> Eq for IndexSet<T, S> {}

impl<T: Hash + Eq, S: BuildHasher> Extend<T> for IndexSet<T, S> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iterable: I) {
        // Upstream runs the map's Extend over `(x, ())` pairs; its
        // reserve (the hint when empty, else half) is written here,
        // because a generic `extend` call over an adapter iterator
        // type does not resolve in lust yet.
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let reserve = if self.map.is_empty() { lower } else { (lower + 1) / 2 };
        self.map.reserve(reserve);
        for x in iter {
            self.map.insert(x, ());
        }
    }
}

impl<T: Hash + Eq, S: BuildHasher + Default> FromIterator<T> for IndexSet<T, S> {
    fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
        // The map's from_iter over `(x, ())` pairs, inlined as in
        // `extend` above.
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut map = IndexMap::with_capacity_and_hasher(lower, S::default());
        for x in iter {
            map.insert(x, ());
        }
        IndexSet { map }
    }
}
