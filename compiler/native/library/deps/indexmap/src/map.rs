// `indexmap::map`: IndexMap, its entry API, and its iterators. Lookups
// keep upstream's shortcuts (an empty map and a one-entry map compare
// without hashing), so a Hash or Eq impl with side effects sees the
// same calls as upstream.

use crate::inner::{Bucket, Core};
use core::hash::{BuildHasher, Hash};
#[cfg(feature = "std")]
use std::collections::hash_map::RandomState;

/// A hash map that keeps insertion order. `S` defaults to `RandomState`
/// only with upstream's `std` feature, as upstream.
#[cfg(feature = "std")]
pub struct IndexMap<K, V, S = RandomState> {
    pub(crate) core: Core<K, V>,
    hash_builder: S,
}

/// A hash map that keeps insertion order (no `std`: no default hasher).
#[cfg(not(feature = "std"))]
pub struct IndexMap<K, V, S> {
    pub(crate) core: Core<K, V>,
    hash_builder: S,
}

#[cfg(feature = "std")]
impl<K, V> IndexMap<K, V> {
    pub fn new() -> Self {
        IndexMap { core: Core::new(), hash_builder: RandomState::new() }
    }

    pub fn with_capacity(n: usize) -> Self {
        IndexMap { core: Core::with_capacity(n), hash_builder: RandomState::new() }
    }
}

impl<K, V, S> IndexMap<K, V, S> {
    pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
        if n == 0 {
            IndexMap { core: Core::new(), hash_builder }
        } else {
            IndexMap { core: Core::with_capacity(n), hash_builder }
        }
    }

    pub fn with_hasher(hash_builder: S) -> Self {
        IndexMap { core: Core::new(), hash_builder }
    }

    pub fn capacity(&self) -> usize {
        self.core.capacity()
    }

    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    pub fn len(&self) -> usize {
        self.core.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { entries: &self.core.entries, front: 0, back: self.core.entries.len() }
    }

    /// All entries as a `Slice`, in order.
    pub fn as_slice(&self) -> &Slice<K, V> {
        Slice::from_slice(self.core.entries.as_slice())
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut { inner: self.core.entries.iter_mut() }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut { inner: self.core.entries.iter_mut() }
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys { inner: self.core.entries.into_iter() }
    }

    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues { inner: self.core.entries.into_iter() }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    pub fn clear(&mut self) {
        self.core.clear();
    }

    pub fn truncate(&mut self, len: usize) {
        self.core.truncate(len);
    }

    pub fn split_off(&mut self, at: usize) -> Self
    where
        S: Clone,
    {
        IndexMap { core: self.core.split_off(at), hash_builder: self.hash_builder.clone() }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.core.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.core.reserve_exact(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.core.shrink_to(0);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.core.shrink_to(min_capacity);
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        self.core.pop()
    }

    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, mut keep: F) {
        self.core.retain_in_order(|k, v| keep(k, v));
    }

    pub fn sort_keys(&mut self)
    where
        K: Ord,
    {
        self.core.entries.sort_by(|a, b| a.key.cmp(&b.key));
        self.core.rebuild_hash_table();
    }

    pub fn sort_by<F: FnMut(&K, &V, &K, &V) -> core::cmp::Ordering>(&mut self, mut cmp: F) {
        self.core.entries.sort_by(|a, b| cmp(&a.key, &a.value, &b.key, &b.value));
        self.core.rebuild_hash_table();
    }

    pub fn sort_by_key<T: Ord, F: FnMut(&K, &V) -> T>(&mut self, mut sort_key: F) {
        self.core.entries.sort_by_key(|b| sort_key(&b.key, &b.value));
        self.core.rebuild_hash_table();
    }

    pub fn sort_unstable_by_key<T: Ord, F: FnMut(&K, &V) -> T>(&mut self, mut sort_key: F) {
        self.core.entries.sort_unstable_by_key(|b| sort_key(&b.key, &b.value));
        self.core.rebuild_hash_table();
    }

    pub fn sort_unstable_keys(&mut self)
    where
        K: Ord,
    {
        self.core.entries.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        self.core.rebuild_hash_table();
    }

    pub fn sort_unstable_by<F: FnMut(&K, &V, &K, &V) -> core::cmp::Ordering>(&mut self, mut cmp: F) {
        self.core.entries.sort_unstable_by(|a, b| cmp(&a.key, &a.value, &b.key, &b.value));
        self.core.rebuild_hash_table();
    }

    pub fn reverse(&mut self) {
        self.core.reverse();
    }

    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        match self.core.entries.get(index) {
            Some(e) => Some((&e.key, &e.value)),
            None => None,
        }
    }

    pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        match self.core.entries.get_mut(index) {
            Some(e) => Some((&e.key, &mut e.value)),
            None => None,
        }
    }

    pub fn first(&self) -> Option<(&K, &V)> {
        self.get_index(0)
    }

    pub fn last(&self) -> Option<(&K, &V)> {
        if self.core.entries.is_empty() { None } else { self.get_index(self.core.entries.len() - 1) }
    }

    pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        self.get_index_mut(0)
    }

    pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.core.entries.is_empty() {
            return None;
        }
        let i = self.core.entries.len() - 1;
        self.get_index_mut(i)
    }

    pub fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        self.core.swap_remove_index(index)
    }

    pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        self.core.shift_remove_index(index)
    }

    pub fn move_index(&mut self, from: usize, to: usize) {
        self.core.move_index(from, to);
    }

    pub fn swap_indices(&mut self, a: usize, b: usize) {
        self.core.swap_indices(a, b);
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> IndexMap<K, V, S> {
    fn hash<Q: ?Sized + Hash>(&self, key: &Q) -> u64 {
        self.hash_builder.hash_one(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert_full(key, value).1
    }

    pub fn insert_full(&mut self, key: K, value: V) -> (usize, Option<V>) {
        let hash = self.hash(&key);
        self.core.insert_full(hash, key, value, |a, b| a == b)
    }

    /// Upstream's entry probes without reserving; a vacant insert then
    /// reserves only if the table is full.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        let hash = self.hash(&key);
        match self.core.find(hash, |k| *k == key) {
            Some(index) => Entry::Occupied(OccupiedEntry { core: &mut self.core, index }),
            None => Entry::Vacant(VacantEntry { core: &mut self.core, hash, key }),
        }
    }

    pub fn contains_key<Q: ?Sized + Hash + Eq>(&self, key: &Q) -> bool
    where
        K: core::borrow::Borrow<Q>,
    {
        self.get_index_of(key).is_some()
    }

    pub fn get<Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<&V>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.get_index_of(key) {
            Some(i) => Some(&self.core.entries[i].value),
            None => None,
        }
    }

    pub fn get_key_value<Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.get_index_of(key) {
            Some(i) => Some((&self.core.entries[i].key, &self.core.entries[i].value)),
            None => None,
        }
    }

    pub fn get_full<Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<(usize, &K, &V)>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.get_index_of(key) {
            Some(i) => Some((i, &self.core.entries[i].key, &self.core.entries[i].value)),
            None => None,
        }
    }

    /// The index of `key`. An empty map answers without hashing, and a
    /// one-entry map compares its key directly, as upstream.
    pub fn get_index_of<Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<usize>
    where
        K: core::borrow::Borrow<Q>,
    {
        let n = self.core.entries.len();
        if n == 0 {
            return None;
        }
        if n == 1 {
            return if key == self.core.entries[0].key.borrow() { Some(0) } else { None };
        }
        let hash = self.hash(key);
        self.core.find(hash, |k| key == k.borrow())
    }

    pub fn get_mut<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.get_index_of(key) {
            Some(i) => Some(&mut self.core.entries[i].value),
            None => None,
        }
    }

    pub fn swap_remove<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<V>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.swap_remove_full(key) {
            Some((_, _, v)) => Some(v),
            None => None,
        }
    }

    pub fn swap_remove_entry<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.swap_remove_full(key) {
            Some((_, k, v)) => Some((k, v)),
            None => None,
        }
    }

    pub fn swap_remove_full<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        K: core::borrow::Borrow<Q>,
    {
        let n = self.core.entries.len();
        if n <= 1 {
            if n == 1 && key == self.core.entries[0].key.borrow() {
                let (k, v) = self.core.pop()?;
                return Some((0, k, v));
            }
            return None;
        }
        let hash = self.hash(key);
        self.core.swap_remove_full(hash, |k| key == k.borrow())
    }

    pub fn shift_remove<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<V>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.shift_remove_full(key) {
            Some((_, _, v)) => Some(v),
            None => None,
        }
    }

    pub fn shift_remove_entry<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: core::borrow::Borrow<Q>,
    {
        match self.shift_remove_full(key) {
            Some((_, k, v)) => Some((k, v)),
            None => None,
        }
    }

    pub fn shift_remove_full<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        K: core::borrow::Borrow<Q>,
    {
        let n = self.core.entries.len();
        if n <= 1 {
            if n == 1 && key == self.core.entries[0].key.borrow() {
                let (k, v) = self.core.pop()?;
                return Some((0, k, v));
            }
            return None;
        }
        let hash = self.hash(key);
        self.core.shift_remove_full(hash, |k| key == k.borrow())
    }

    /// Upstream deprecates `remove`; it is `swap_remove`.
    pub fn remove<Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<V>
    where
        K: core::borrow::Borrow<Q>,
    {
        self.swap_remove(key)
    }
}

/// An entry of `IndexMap::entry`.
pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

/// An entry whose key is in the map.
pub struct OccupiedEntry<'a, K, V> {
    core: &'a mut Core<K, V>,
    index: usize,
}

/// An entry whose key is not in the map yet.
pub struct VacantEntry<'a, K, V> {
    core: &'a mut Core<K, V>,
    hash: u64,
    key: K,
}

impl<'a, K, V> Entry<'a, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(default),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, call: F) -> &'a mut V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(call()),
        }
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(V::default()),
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(e) = &mut self {
            f(e.get_mut());
        }
        self
    }

    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(e) => e.key(),
            Entry::Vacant(e) => e.key(),
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Entry::Occupied(e) => e.index,
            Entry::Vacant(e) => e.core.len(),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.core.entries[self.index].key
    }

    pub fn get(&self) -> &V {
        &self.core.entries[self.index].value
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.core.entries[self.index].value
    }

    pub fn into_mut(self) -> &'a mut V {
        &mut self.core.entries[self.index].value
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn insert(&mut self, value: V) -> V {
        core::mem::replace(self.get_mut(), value)
    }

    pub fn swap_remove(self) -> V {
        match self.core.swap_remove_index(self.index) {
            Some((_, v)) => v,
            None => panic!("index out of bounds"),
        }
    }

    pub fn shift_remove(self) -> V {
        match self.core.shift_remove_index(self.index) {
            Some((_, v)) => v,
            None => panic!("index out of bounds"),
        }
    }
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn index(&self) -> usize {
        self.core.len()
    }

    pub fn insert(self, value: V) -> &'a mut V {
        let i = self.core.insert_unique(self.hash, self.key, value);
        &mut self.core.entries[i].value
    }
}

/// The entries in order, as `(&K, &V)`.
pub struct Iter<'a, K, V> {
    entries: &'a [Bucket<K, V>],
    front: usize,
    back: usize,
}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Iter { entries: self.entries, front: self.front, back: self.back }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        if self.front == self.back {
            return None;
        }
        let e = &self.entries[self.front];
        self.front += 1;
        Some((&e.key, &e.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.back - self.front;
        (n, Some(n))
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        let e = &self.entries[self.back];
        Some((&e.key, &e.value))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

/// The entries in order, as `(&K, &mut V)`.
pub struct IterMut<'a, K, V> {
    inner: core::slice::IterMut<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        match self.inner.next() {
            Some(b) => Some((&b.key, &mut b.value)),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        match self.inner.next_back() {
            Some(b) => Some((&b.key, &mut b.value)),
            None => None,
        }
    }
}

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}

/// The values in order, mutable.
pub struct ValuesMut<'a, K, V> {
    inner: core::slice::IterMut<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<&'a mut V> {
        match self.inner.next() {
            Some(b) => Some(&mut b.value),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a mut V> {
        match self.inner.next_back() {
            Some(b) => Some(&mut b.value),
            None => None,
        }
    }
}

impl<'a, K, V> ExactSizeIterator for ValuesMut<'a, K, V> {}

/// The keys in order, by value.
pub struct IntoKeys<K, V> {
    inner: std::vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        match self.inner.next() {
            Some(b) => Some(b.key),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    fn next_back(&mut self) -> Option<K> {
        match self.inner.next_back() {
            Some(b) => Some(b.key),
            None => None,
        }
    }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {}

/// The values in order, by value.
pub struct IntoValues<K, V> {
    inner: std::vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<V> {
        match self.inner.next() {
            Some(b) => Some(b.value),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<V> {
        match self.inner.next_back() {
            Some(b) => Some(b.value),
            None => None,
        }
    }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {}

/// The keys in order.
pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Self {
        Keys { inner: self.inner.clone() }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        match self.inner.next() {
            Some((k, _)) => Some(k),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        match self.inner.next_back() {
            Some((k, _)) => Some(k),
            None => None,
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {}

/// The values in order.
pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Self {
        Values { inner: self.inner.clone() }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        match self.inner.next() {
            Some((_, v)) => Some(v),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        match self.inner.next_back() {
            Some((_, v)) => Some(v),
            None => None,
        }
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {}

impl<'a, K, V, S> IntoIterator for &'a IndexMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut IndexMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}

/// The entries in order, by value.
pub struct IntoIter<K, V> {
    inner: std::vec::IntoIter<Bucket<K, V>>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        match self.inner.next() {
            Some(b) => Some((b.key, b.value)),
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<(K, V)> {
        match self.inner.next_back() {
            Some(b) => Some((b.key, b.value)),
            None => None,
        }
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {}

/// A dynamically-sized slice of an `IndexMap`'s entries (upstream's
/// `map::Slice`, `#[repr(transparent)]` over `[Bucket<K, V>]`). Unlike
/// `IndexMap`, its `PartialEq` and `Eq` consider order. lust makes the
/// reference from the entries by its shipped-source coercion of a
/// reference to an unsized newtype's field (upstream: a pointer cast).
#[repr(transparent)]
pub struct Slice<K, V> {
    pub(crate) entries: [Bucket<K, V>],
}

impl<K, V> Slice<K, V> {
    pub(crate) fn from_slice(entries: &[Bucket<K, V>]) -> &Self {
        entries
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        match self.entries.get(index) {
            Some(b) => Some((&b.key, &b.value)),
            None => None,
        }
    }

    pub const fn first(&self) -> Option<(&K, &V)> {
        match self.entries.first() {
            Some(b) => Some((&b.key, &b.value)),
            None => None,
        }
    }

    pub const fn last(&self) -> Option<(&K, &V)> {
        match self.entries.last() {
            Some(b) => Some((&b.key, &b.value)),
            None => None,
        }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter { entries: &self.entries, front: 0, back: self.entries.len() }
    }

    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }
}

impl<'a, K, V> IntoIterator for &'a Slice<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<K: core::fmt::Debug, V: core::fmt::Debug> core::fmt::Debug for Slice<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Upstream passes `self` (its `IntoIterator` is `iter`).
        f.debug_list().entries(self.iter()).finish()
    }
}

/// Upstream's `slice_eq`: lengths, then each pair by index, in order.
impl<K, V, K2, V2> PartialEq<Slice<K2, V2>> for Slice<K, V>
where
    K: PartialEq<K2>,
    V: PartialEq<V2>,
{
    fn eq(&self, other: &Slice<K2, V2>) -> bool {
        if self.entries.len() != other.entries.len() {
            return false;
        }
        for i in 0..self.entries.len() {
            if !(self.entries[i].key == other.entries[i].key && self.entries[i].value == other.entries[i].value) {
                return false;
            }
        }
        true
    }
}

impl<K: Eq, V: Eq> Eq for Slice<K, V> {}

impl<K, V, S> IntoIterator for IndexMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter { inner: self.core.entries.into_iter() }
    }
}

impl<K: Clone, V: Clone, S: Clone> Clone for IndexMap<K, V, S> {
    fn clone(&self) -> Self {
        IndexMap { core: self.core.clone(), hash_builder: self.hash_builder.clone() }
    }
}

impl<K, V, S: Default> Default for IndexMap<K, V, S> {
    fn default() -> Self {
        IndexMap { core: Core::new(), hash_builder: S::default() }
    }
}

impl<K: core::fmt::Debug, V: core::fmt::Debug, S> core::fmt::Debug for IndexMap<K, V, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Upstream's `entries(self.iter())`, one entry at a time (lust's
        // DebugMap has no `entries` yet; the output is the same).
        let mut map = f.debug_map();
        let entries = &self.core.entries;
        let mut i = 0;
        while i < entries.len() {
            map.entry(&entries[i].key, &entries[i].value);
            i += 1;
        }
        map.finish()
    }
}

/// Two maps are equal when they hold the same pairs, in any order.
impl<K: Hash + Eq, V1: PartialEq<V2>, V2, S1: BuildHasher, S2: BuildHasher> PartialEq<IndexMap<K, V2, S2>> for IndexMap<K, V1, S1> {
    fn eq(&self, other: &IndexMap<K, V2, S2>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (key, value) in self.iter() {
            match other.get(key) {
                Some(v) => {
                    if *value != *v {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }
}

impl<K: Hash + Eq, V: Eq, S: BuildHasher> Eq for IndexMap<K, V, S> {}

impl<K: Hash + Eq, V, S: BuildHasher> Extend<(K, V)> for IndexMap<K, V, S> {
    /// Upstream reserves the whole hint when the map is empty, else half.
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iterable: I) {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let reserve = if self.is_empty() { lower } else { (lower + 1) / 2 };
        self.reserve(reserve);
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> FromIterator<(K, V)> for IndexMap<K, V, S> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iterable: I) -> Self {
        // Upstream calls `map.extend(iter)`; on a fresh map with
        // capacity `lower` that reserve is a no-op, so the loop is
        // the same, and a generic `extend` call whose argument is
        // an `IntoIter` projection does not resolve in lust yet.
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut map = IndexMap::with_capacity_and_hasher(lower, S::default());
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

impl<K: Hash + Eq + core::borrow::Borrow<Q>, Q: ?Sized + Hash + Eq, V, S: BuildHasher> core::ops::Index<&Q> for IndexMap<K, V, S> {
    type Output = V;

    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K: Hash + Eq + core::borrow::Borrow<Q>, Q: ?Sized + Hash + Eq, V, S: BuildHasher> core::ops::IndexMut<&Q> for IndexMap<K, V, S> {
    fn index_mut(&mut self, key: &Q) -> &mut V {
        self.get_mut(key).expect("no entry found for key")
    }
}

/// The value at a position, as upstream's `Index<usize>` (same panic text).
impl<K, V, S> core::ops::Index<usize> for IndexMap<K, V, S> {
    type Output = V;

    fn index(&self, index: usize) -> &V {
        if let Some((_, value)) = self.get_index(index) {
            value
        } else {
            panic!("index out of bounds: the len is {len} but the index is {index}", len = self.len());
        }
    }
}

impl<K, V, S> core::ops::IndexMut<usize> for IndexMap<K, V, S> {
    fn index_mut(&mut self, index: usize) -> &mut V {
        let len: usize = self.len();
        if let Some((_, value)) = self.get_index_mut(index) {
            value
        } else {
            panic!("index out of bounds: the len is {len} but the index is {index}");
        }
    }
}
