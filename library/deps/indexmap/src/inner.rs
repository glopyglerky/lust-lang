// The core of IndexMap and IndexSet: entries in insertion order plus the
// hash index (indices.rs). A port of indexmap 2.14.0's src/inner.rs: each
// operation touches the index the way upstream does (which buckets it
// erases, when it rebuilds, how it reserves), so capacities and the
// candidates `eq` sees are upstream's.

use crate::indices::Indices;

/// One entry: the key's hash (kept, so growth never hashes again), the
/// key, and the value.
#[derive(Clone)]
pub(crate) struct Bucket<K, V> {
    pub(crate) hash: u64,
    pub(crate) key: K,
    pub(crate) value: V,
}

pub(crate) struct Core<K, V> {
    pub(crate) indices: Indices,
    pub(crate) entries: Vec<Bucket<K, V>>,
}

/// Upstream caps the entries' soft reserve at isize::MAX / size_of of a
/// bucket; no reachable table gets there, so the cap here is isize::MAX.
const MAX_ENTRIES_CAPACITY: usize = 9223372036854775807;

/// Inserts every entry's index in order, as upstream's
/// insert_bulk_no_grow: index `indices.len()` for each.
fn insert_bulk_no_grow<K, V>(indices: &mut Indices, entries: &[Bucket<K, V>]) {
    assert!(indices.capacity() - indices.len() >= entries.len());
    let mut i = 0;
    while i < entries.len() {
        let n = indices.len();
        indices.insert_unique(entries[i].hash, n, |_| 0);
        i += 1;
    }
}

/// Erases the bucket that holds entry index `index`.
fn erase_index(indices: &mut Indices, hash: u64, index: usize) {
    if let Some(bucket) = indices.find(hash, |i| i == index) {
        indices.erase(bucket);
    }
}

/// Moves the bucket for entry index `old` to hold `new`.
fn update_index(indices: &mut Indices, hash: u64, old: usize, new: usize) {
    match indices.find(hash, |i| i == old) {
        Some(bucket) => indices.set(bucket, new),
        None => panic!("index not found"),
    }
}

impl<K: Clone, V: Clone> Clone for Core<K, V> {
    fn clone(&self) -> Self {
        let mut new = Core::new();
        new.indices = self.indices.clone();
        if new.entries.capacity() < self.entries.len() {
            let additional = self.entries.len() - new.entries.len();
            new.reserve_entries(additional);
        }
        let mut i = 0;
        while i < self.entries.len() {
            new.entries.push(self.entries[i].clone());
            i += 1;
        }
        new
    }
}

impl<K, V> Core<K, V> {
    pub(crate) fn new() -> Self {
        Core { indices: Indices::new(), entries: Vec::new() }
    }

    pub(crate) fn with_capacity(n: usize) -> Self {
        Core { indices: Indices::with_capacity(n), entries: Vec::with_capacity(n) }
    }

    pub(crate) fn len(&self) -> usize {
        self.indices.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        let a = self.indices.capacity();
        let b = self.entries.capacity();
        if a < b { a } else { b }
    }

    pub(crate) fn clear(&mut self) {
        self.indices.clear();
        self.entries.clear();
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.erase_indices(len, self.entries.len());
            self.entries.truncate(len);
        }
    }

    pub(crate) fn split_off(&mut self, at: usize) -> Self {
        let len = self.entries.len();
        assert!(at <= len, "index out of bounds: the len is {len} but the index is {at}. Expected index <= len");
        self.erase_indices(at, self.entries.len());
        let entries = self.entries.split_off(at);
        let mut indices = Indices::with_capacity(entries.len());
        insert_bulk_no_grow(&mut indices, &entries);
        Core { indices, entries }
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        let entries = &self.entries;
        self.indices.reserve(additional, |i| entries[i].hash);
        if additional > self.entries.capacity() - self.entries.len() {
            self.reserve_entries(additional);
        }
    }

    pub(crate) fn reserve_exact(&mut self, additional: usize) {
        let entries = &self.entries;
        self.indices.reserve(additional, |i| entries[i].hash);
        self.entries.reserve_exact(additional);
    }

    pub(crate) fn shrink_to(&mut self, min_capacity: usize) {
        let entries = &self.entries;
        self.indices.shrink_to(min_capacity, |i| entries[i].hash);
        self.entries.shrink_to(min_capacity);
    }

    pub(crate) fn pop(&mut self) -> Option<(K, V)> {
        match self.entries.pop() {
            Some(entry) => {
                let last = self.entries.len();
                erase_index(&mut self.indices, entry.hash, last);
                Some((entry.key, entry.value))
            }
            None => None,
        }
    }

    /// The entry index whose key `is_match` accepts, probing for `hash`.
    pub(crate) fn find<F: FnMut(&K) -> bool>(&self, hash: u64, mut is_match: F) -> Option<usize> {
        let entries = &self.entries;
        match self.indices.find(hash, |i| is_match(&entries[i].key)) {
            Some(bucket) => Some(self.indices.get(bucket)),
            None => None,
        }
    }

    fn push_entry(&mut self, hash: u64, key: K, value: V) {
        if self.entries.len() == self.entries.capacity() {
            self.reserve_entries(1);
        }
        self.entries.push(Bucket { hash, key, value });
    }

    /// Inserts or replaces the value; `is_match` compares the new key with
    /// a stored one. Returns the entry index and the old value.
    pub(crate) fn insert_full<F: FnMut(&K, &K) -> bool>(&mut self, hash: u64, key: K, value: V, mut is_match: F) -> (usize, Option<V>) {
        let found = {
            let entries = &self.entries;
            let k = &key;
            self.indices.entry(hash, |i| is_match(k, &entries[i].key), |i| entries[i].hash)
        };
        match found {
            Ok(bucket) => {
                let i = self.indices.get(bucket);
                (i, Some(core::mem::replace(&mut self.entries[i].value, value)))
            }
            Err(slot) => {
                let i = self.entries.len();
                self.indices.insert_at(hash, slot, i);
                self.push_entry(hash, key, value);
                (i, None)
            }
        }
    }

    pub(crate) fn shift_remove_full<F: FnMut(&K) -> bool>(&mut self, hash: u64, mut is_match: F) -> Option<(usize, K, V)> {
        let bucket = {
            let entries = &self.entries;
            self.indices.find(hash, |i| is_match(&entries[i].key))?
        };
        let index = self.indices.get(bucket);
        self.indices.erase(bucket);
        let (key, value) = self.shift_remove_finish(index);
        Some((index, key, value))
    }

    pub(crate) fn swap_remove_full<F: FnMut(&K) -> bool>(&mut self, hash: u64, mut is_match: F) -> Option<(usize, K, V)> {
        let bucket = {
            let entries = &self.entries;
            self.indices.find(hash, |i| is_match(&entries[i].key))?
        };
        let index = self.indices.get(bucket);
        self.indices.erase(bucket);
        let (key, value) = self.swap_remove_finish(index);
        Some((index, key, value))
    }

    /// Drops the index entries for `start..end` and moves later ones down.
    fn erase_indices(&mut self, start: usize, end: usize) {
        let erased = end - start;
        let shifted = self.entries.len() - end;
        let half_capacity = self.indices.capacity() / 2;
        if erased == 0 {
            // Nothing to do.
        } else if start + shifted < half_capacity && start < erased {
            self.indices.clear();
            insert_bulk_no_grow(&mut self.indices, &self.entries[..start]);
            insert_bulk_no_grow(&mut self.indices, &self.entries[end..]);
        } else if erased + shifted < half_capacity {
            let mut i = start;
            while i < end {
                erase_index(&mut self.indices, self.entries[i].hash, i);
                i += 1;
            }
            let mut old = end;
            while old < self.entries.len() {
                update_index(&mut self.indices, self.entries[old].hash, old, old - erased);
                old += 1;
            }
        } else {
            self.indices.retain(|i| {
                if *i >= end {
                    *i -= erased;
                    true
                } else {
                    *i < start
                }
            });
        }
    }

    pub(crate) fn retain_in_order<F: FnMut(&mut K, &mut V) -> bool>(&mut self, mut keep: F) {
        let old = core::mem::take(&mut self.entries);
        let cap = old.capacity();
        let mut kept: Vec<Bucket<K, V>> = Vec::with_capacity(cap);
        for mut entry in old {
            if keep(&mut entry.key, &mut entry.value) {
                kept.push(entry);
            }
        }
        self.entries = kept;
        if self.entries.len() < self.indices.len() {
            self.rebuild_hash_table();
        }
    }

    pub(crate) fn rebuild_hash_table(&mut self) {
        self.indices.clear();
        insert_bulk_no_grow(&mut self.indices, &self.entries);
    }

    pub(crate) fn reverse(&mut self) {
        self.entries.reverse();
        let len = self.entries.len();
        self.indices.for_each_mut(|i| *i = len - *i - 1);
    }

    fn reserve_entries(&mut self, additional: usize) {
        let ic = self.indices.capacity();
        let try_capacity = if ic < MAX_ENTRIES_CAPACITY { ic } else { MAX_ENTRIES_CAPACITY };
        let try_add = try_capacity - self.entries.len();
        if try_add > additional {
            self.entries.reserve_exact(try_add);
            return;
        }
        self.entries.reserve_exact(additional);
    }

    /// Appends a key known to be absent.
    pub(crate) fn insert_unique(&mut self, hash: u64, key: K, value: V) -> usize {
        let i = self.indices.len();
        let entries = &self.entries;
        self.indices.insert_unique(hash, i, |j| entries[j].hash);
        self.push_entry(hash, key, value);
        i
    }

    /// Inserts a key known to be absent at `index`, moving later ones up.
    pub(crate) fn shift_insert_unique(&mut self, index: usize, hash: u64, key: K, value: V) {
        let end = self.indices.len();
        assert!(index <= end);
        self.increment_indices(index, end);
        let entries = &self.entries;
        self.indices.insert_unique(hash, index, |i| {
            let i = if i < index { i } else { i - 1 };
            entries[i].hash
        });
        if self.entries.len() == self.entries.capacity() {
            self.reserve_entries(1);
        }
        self.entries.insert(index, Bucket { hash, key, value });
    }

    pub(crate) fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        if index >= self.entries.len() {
            return None;
        }
        erase_index(&mut self.indices, self.entries[index].hash, index);
        Some(self.shift_remove_finish(index))
    }

    fn shift_remove_finish(&mut self, index: usize) -> (K, V) {
        self.decrement_indices(index + 1, self.entries.len());
        let entry = self.entries.remove(index);
        (entry.key, entry.value)
    }

    pub(crate) fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        if index >= self.entries.len() {
            return None;
        }
        erase_index(&mut self.indices, self.entries[index].hash, index);
        Some(self.swap_remove_finish(index))
    }

    fn swap_remove_finish(&mut self, index: usize) -> (K, V) {
        let entry = self.entries.swap_remove(index);
        if index < self.entries.len() {
            let last = self.entries.len();
            update_index(&mut self.indices, self.entries[index].hash, last, index);
        }
        (entry.key, entry.value)
    }

    fn decrement_indices(&mut self, start: usize, end: usize) {
        if end - start > self.indices.capacity() / 2 {
            self.indices.for_each_mut(|i| {
                if start <= *i && *i < end {
                    *i -= 1;
                }
            });
        } else {
            let mut i = start;
            while i < end {
                update_index(&mut self.indices, self.entries[i].hash, i, i - 1);
                i += 1;
            }
        }
    }

    fn increment_indices(&mut self, start: usize, end: usize) {
        if end - start > self.indices.capacity() / 2 {
            self.indices.for_each_mut(|i| {
                if start <= *i && *i < end {
                    *i += 1;
                }
            });
        } else {
            let mut i = end;
            while i > start {
                i -= 1;
                update_index(&mut self.indices, self.entries[i].hash, i, i + 1);
            }
        }
    }

    pub(crate) fn move_index(&mut self, from: usize, to: usize) {
        let from_hash = self.entries[from].hash;
        if from != to {
            let _ = &self.entries[to];
            let bucket = match self.indices.find(from_hash, |i| i == from) {
                Some(b) => b,
                None => panic!("index not found"),
            };
            if from < to {
                self.decrement_indices(from + 1, to + 1);
                // `entries[from..=to].rotate_left(1)` as adjacent swaps:
                // the same order, and no element is dropped or cloned.
                let mut i = from;
                while i < to {
                    self.entries.swap(i, i + 1);
                    i += 1;
                }
            } else {
                self.increment_indices(to, from);
                // `entries[to..=from].rotate_right(1)`, the same way.
                let mut i = from;
                while i > to {
                    self.entries.swap(i - 1, i);
                    i -= 1;
                }
            }
            self.indices.set(bucket, to);
        }
    }

    pub(crate) fn swap_indices(&mut self, a: usize, b: usize) {
        if a == b && a < self.entries.len() {
            return;
        }
        let ha = self.entries[a].hash;
        let hb = self.entries[b].hash;
        let ba = self.indices.find(ha, |i| i == a);
        let bb = self.indices.find(hb, |i| i == b);
        match (ba, bb) {
            (Some(x), Some(y)) => {
                self.indices.set(x, b);
                self.indices.set(y, a);
                self.entries.swap(a, b);
            }
            _ => panic!("indices not found"),
        }
    }
}
