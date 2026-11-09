//! Bench stand-in for the `smallvec` crate: a copy of lust's adapter
//! (library/deps/smallvec/src/lib.rs, smallvec 1.15.1 shape) mounted as
//! `crate::smallvec`. One change: `smallvec!` is a module macro
//! (`pub(crate) use`), not `#[macro_export]`, so `smallvec::smallvec![..]`
//! paths (rewritten to `crate::smallvec::smallvec!`) resolve.

// Lust adapter for smallvec 1.15.1 (docs/ARCH.md section 3.6): the real
// public shape, `SmallVec<A>` with `A: Array` (callers write
// `SmallVec<[T; N]>`). The items live in a Vec; the adapter keeps
// upstream's logical capacity: `inline_size()` until the first spill,
// then the heap capacity upstream would have chosen (power-of-two growth
// on push and reserve, exact on reserve_exact and shrink_to_fit, back
// inline when it fits). Drop orders follow upstream too: truncate,
// clear, retain, and dedup drop from the back, after all calls. The
// layout differs from upstream's (no inline buffer), so layout queries
// on SmallVec reject (L0652). Not here (they reject at the use):
// insert_many, raw pointers and parts, from_buf, the const forms,
// write, serde, arbitrary, and drain_filter.

use core::ops::{Deref, DerefMut};

/// The inline storage a SmallVec names: `[T; N]` for upstream's sizes.
pub unsafe trait Array {
    type Item;
    fn size() -> usize;
}

macro_rules! impl_array {
    ($($size:expr),+) => {
        $(
            unsafe impl<T> Array for [T; $size] {
                type Item = T;
                fn size() -> usize {
                    $size
                }
            }
        )+
    };
}

// Upstream's sizes without the const_generics feature.
impl_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 36, 0x40,
    0x60, 0x80, 0x100, 0x200, 0x400, 0x600, 0x800, 0x1000, 0x2000, 0x4000, 0x6000, 0x8000, 0x10000, 0x20000, 0x40000, 0x60000, 0x80000,
    0x10_0000
);

/// A vector that keeps up to `A::size()` items inline (in upstream).
pub struct SmallVec<A: Array> {
    /// The heap capacity once spilled (greater than the inline size);
    /// 0 while inline. The Vec's own capacity equals it when spilled.
    heap_cap: usize,
    items: Vec<A::Item>,
}

/// The smallest power of two at or above `n`, as
/// `usize::checked_next_power_of_two(n).expect("capacity overflow")`.
fn next_power_of_two(n: usize) -> usize {
    let mut p: usize = 1;
    while p < n {
        p = match p.checked_mul(2) {
            Some(q) => q,
            None => panic!("capacity overflow"),
        };
    }
    p
}

/// Vec's smallest non-zero capacity for an element size.
fn min_non_zero_cap(elem_size: usize) -> usize {
    if elem_size == 1 {
        8
    } else if elem_size <= 1024 {
        4
    } else {
        1
    }
}

impl<A: Array> SmallVec<A> {
    pub fn new() -> SmallVec<A> {
        SmallVec { heap_cap: 0, items: Vec::new() }
    }

    pub fn with_capacity(n: usize) -> Self {
        let mut v = SmallVec::new();
        v.reserve_exact(n);
        v
    }

    /// Takes the Vec's buffer when it is larger than the inline size;
    /// else the items move inline.
    pub fn from_vec(vec: Vec<A::Item>) -> SmallVec<A> {
        if vec.capacity() <= SmallVec::<A>::inline_capacity() {
            SmallVec { heap_cap: 0, items: vec }
        } else {
            let cap = vec.capacity();
            SmallVec { heap_cap: cap, items: vec }
        }
    }

    fn inline_capacity() -> usize {
        if core::mem::size_of::<A::Item>() > 0 { A::size() } else { usize::MAX }
    }

    pub fn inline_size(&self) -> usize {
        SmallVec::<A>::inline_capacity()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn capacity(&self) -> usize {
        if self.spilled() { self.heap_cap } else { SmallVec::<A>::inline_capacity() }
    }

    pub fn spilled(&self) -> bool {
        self.heap_cap > SmallVec::<A>::inline_capacity()
    }

    pub fn push(&mut self, value: A::Item) {
        if self.items.len() == self.capacity() {
            self.reserve_one_unchecked();
        }
        self.items.push(value);
    }

    pub fn pop(&mut self) -> Option<A::Item> {
        self.items.pop()
    }

    /// Re-allocates to `new_cap` items: back inline when it fits, else
    /// a heap buffer of exactly `new_cap`.
    pub fn grow(&mut self, new_cap: usize) {
        let len = self.items.len();
        assert!(new_cap >= len);
        if new_cap <= SmallVec::<A>::inline_capacity() {
            if !self.spilled() {
                return;
            }
            self.heap_cap = 0;
            self.items.shrink_to_fit();
        } else if new_cap != self.capacity() {
            if new_cap > self.items.capacity() {
                self.items.reserve_exact(new_cap - len);
            } else {
                self.items.shrink_to(new_cap);
            }
            self.heap_cap = new_cap;
        }
    }

    fn reserve_one_unchecked(&mut self) {
        let new_cap = match self.items.len().checked_add(1) {
            Some(n) => next_power_of_two(n),
            None => panic!("capacity overflow"),
        };
        self.grow(new_cap);
    }

    pub fn reserve(&mut self, additional: usize) {
        let len = self.items.len();
        if self.capacity() - len >= additional {
            return;
        }
        let new_cap = match len.checked_add(additional) {
            Some(n) => next_power_of_two(n),
            None => panic!("capacity overflow"),
        };
        self.grow(new_cap);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let len = self.items.len();
        if self.capacity() - len >= additional {
            return;
        }
        let new_cap = match len.checked_add(additional) {
            Some(n) => n,
            None => panic!("capacity overflow"),
        };
        self.grow(new_cap);
    }

    pub fn shrink_to_fit(&mut self) {
        if !self.spilled() {
            return;
        }
        let len = self.items.len();
        if self.inline_size() >= len {
            self.heap_cap = 0;
            self.items.shrink_to_fit();
        } else if self.capacity() > len {
            self.grow(len);
        }
    }

    /// Drops the items past `len`, last first.
    pub fn truncate(&mut self, len: usize) {
        while len < self.items.len() {
            self.items.pop();
        }
    }

    pub fn as_slice(&self) -> &[A::Item] {
        self.items.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [A::Item] {
        self.items.as_mut_slice()
    }

    pub fn swap_remove(&mut self, index: usize) -> A::Item {
        let len = self.items.len();
        self.items.swap(len - 1, index);
        match self.items.pop() {
            Some(x) => x,
            None => panic!("swap_remove on an empty SmallVec"),
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Removes `range` and yields its items. The bounds checks and their
    /// panics are upstream's. When the Drain drops, the items it did not
    /// yield drop front to back and the tail moves down, as upstream's
    /// Drain does; std's `vec::Drain` does both the same way.
    pub fn drain<R: core::ops::RangeBounds<usize>>(&mut self, range: R) -> Drain<'_, A> {
        use core::ops::Bound::*;
        let len = self.len();
        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n.checked_add(1).expect("Range start out of bounds"),
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(&n) => n.checked_add(1).expect("Range end out of bounds"),
            Excluded(&n) => n,
            Unbounded => len,
        };
        assert!(start <= end);
        assert!(end <= len);
        Drain { inner: self.items.drain(start..end) }
    }

    pub fn remove(&mut self, index: usize) -> A::Item {
        let len = self.items.len();
        assert!(index < len);
        self.items.remove(index)
    }

    pub fn insert(&mut self, index: usize, element: A::Item) {
        if self.items.len() == self.capacity() {
            self.reserve_one_unchecked();
        }
        if index > self.items.len() {
            panic!("index exceeds length");
        }
        self.items.insert(index, element);
    }

    /// The items as a Vec: the heap buffer itself when spilled, else a
    /// new Vec sized as `collect` sizes it.
    pub fn into_vec(self) -> Vec<A::Item> {
        if self.spilled() {
            return self.items;
        }
        let len = self.items.len();
        if len == 0 {
            return Vec::new();
        }
        let min = min_non_zero_cap(core::mem::size_of::<A::Item>());
        let mut out = Vec::with_capacity(if min > len { min } else { len });
        for x in self.items {
            out.push(x);
        }
        out
    }

    /// Keeps the items `f` accepts, in order; the rest move to the end and
    /// drop after every call, last first, as upstream.
    pub fn retain<F: FnMut(&mut A::Item) -> bool>(&mut self, mut f: F) {
        let mut del = 0;
        let len = self.items.len();
        let mut i = 0;
        while i < len {
            if !f(&mut self.items[i]) {
                del += 1;
            } else if del > 0 {
                self.items.swap(i - del, i);
            }
            i += 1;
        }
        self.truncate(len - del);
    }

    pub fn retain_mut<F: FnMut(&mut A::Item) -> bool>(&mut self, f: F) {
        self.retain(f)
    }

    pub fn dedup(&mut self)
    where
        A::Item: PartialEq<A::Item>,
    {
        self.dedup_by(|a, b| a == b);
    }

    /// Upstream's dedup: duplicates move to the end, then truncate drops
    /// them last first.
    pub fn dedup_by<F: FnMut(&mut A::Item, &mut A::Item) -> bool>(&mut self, mut same_bucket: F) {
        let len = self.items.len();
        if len <= 1 {
            return;
        }
        let mut w: usize = 1;
        let mut r = 1;
        while r < len {
            // Item r leaves the Vec for the call (a move, not observable)
            // so both can be borrowed mutably; then it goes back.
            let mut cur = self.items.remove(r);
            let same = same_bucket(&mut cur, &mut self.items[w - 1]);
            self.items.insert(r, cur);
            if !same {
                if r != w {
                    self.items.swap(r, w);
                }
                w += 1;
            }
            r += 1;
        }
        self.truncate(w);
    }

    pub fn dedup_by_key<K: PartialEq, F: FnMut(&mut A::Item) -> K>(&mut self, mut key: F) {
        self.dedup_by(|a, b| key(a) == key(b));
    }
}

impl<A: Array> SmallVec<A>
where
    A::Item: Copy,
{
    pub fn from_slice(slice: &[A::Item]) -> Self {
        let len = slice.len();
        if len <= SmallVec::<A>::inline_capacity() {
            SmallVec { heap_cap: 0, items: slice.to_vec() }
        } else {
            let b = slice.to_vec();
            let cap = b.capacity();
            SmallVec { heap_cap: cap, items: b }
        }
    }

    pub fn insert_from_slice(&mut self, index: usize, slice: &[A::Item]) {
        self.reserve(slice.len());
        let len = self.items.len();
        assert!(index <= len);
        let mut i = 0;
        while i < slice.len() {
            self.items.insert(index + i, slice[i]);
            i += 1;
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[A::Item]) {
        let len = self.items.len();
        self.insert_from_slice(len, slice);
    }
}

impl<A: Array> SmallVec<A>
where
    A::Item: Clone,
{
    /// Grows with clones of `value` (and drops `value` last), or truncates.
    pub fn resize(&mut self, len: usize, value: A::Item) {
        let old_len = self.items.len();
        if len > old_len {
            let n = len - old_len;
            self.reserve(n);
            let mut i = 0;
            while i < n {
                self.push(value.clone());
                i += 1;
            }
        } else {
            self.truncate(len);
        }
    }

    /// `n` items equal to `elem`: upstream clones `n` times and drops
    /// `elem` inline; past the inline size it builds `vec![elem; n]`
    /// (n - 1 clones, then `elem` itself).
    pub fn from_elem(elem: A::Item, n: usize) -> Self {
        if n > SmallVec::<A>::inline_capacity() {
            let mut v: Vec<A::Item> = Vec::with_capacity(n);
            let mut i = 1;
            while i < n {
                v.push(elem.clone());
                i += 1;
            }
            v.push(elem);
            SmallVec::from_vec(v)
        } else {
            let mut v = SmallVec::new();
            let mut i = 0;
            while i < n {
                v.items.push(elem.clone());
                i += 1;
            }
            v
        }
    }
}

impl<A: Array> Deref for SmallVec<A> {
    type Target = [A::Item];

    fn deref(&self) -> &[A::Item] {
        self.items.as_slice()
    }
}

impl<A: Array> DerefMut for SmallVec<A> {
    fn deref_mut(&mut self) -> &mut [A::Item] {
        self.items.as_mut_slice()
    }
}

impl<A: Array> Default for SmallVec<A> {
    fn default() -> SmallVec<A> {
        SmallVec::new()
    }
}

/// A clone holds the same items, sized as `collect` sizes it (upstream:
/// `slice.iter().cloned().collect()`).
impl<A: Array> Clone for SmallVec<A>
where
    A::Item: Clone,
{
    fn clone(&self) -> SmallVec<A> {
        let mut v = SmallVec::new();
        v.extend_cloned(self.items.as_slice());
        v
    }
}

impl<A: Array> SmallVec<A>
where
    A::Item: Clone,
{
    fn extend_cloned(&mut self, src: &[A::Item]) {
        self.reserve(src.len());
        let mut i = 0;
        while i < src.len() {
            self.push(src[i].clone());
            i += 1;
        }
    }
}

/// Upstream reserves the size hint's lower bound, fills the capacity it
/// has, then pushes the rest one at a time.
impl<A: Array> Extend<A::Item> for SmallVec<A> {
    fn extend<I: IntoIterator<Item = A::Item>>(&mut self, iterable: I) {
        let mut iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        self.reserve(lower);
        let cap = self.capacity();
        while self.items.len() < cap {
            match iter.next() {
                Some(x) => self.items.push(x),
                None => return,
            }
        }
        for x in iter {
            self.push(x);
        }
    }
}

impl<A: Array> FromIterator<A::Item> for SmallVec<A> {
    fn from_iter<I: IntoIterator<Item = A::Item>>(iterable: I) -> SmallVec<A> {
        // The Extend body inlined: a generic `extend` call with an
        // `Item = A::Item` binding does not resolve in lust yet.
        let mut v = SmallVec::new();
        let mut iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        v.reserve(lower);
        let cap = v.capacity();
        while v.items.len() < cap {
            match iter.next() {
                Some(x) => v.items.push(x),
                None => return v,
            }
        }
        for x in iter {
            v.push(x);
        }
        v
    }
}

impl<A: Array> From<Vec<A::Item>> for SmallVec<A> {
    fn from(vec: Vec<A::Item>) -> SmallVec<A> {
        SmallVec::from_vec(vec)
    }
}

impl<A: Array> core::fmt::Debug for SmallVec<A>
where
    A::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();
        let mut i = 0;
        while i < self.items.len() {
            list.entry(&self.items[i]);
            i += 1;
        }
        list.finish()
    }
}

impl<A: Array, B: Array> PartialEq<SmallVec<B>> for SmallVec<A>
where
    A::Item: PartialEq<B::Item>,
{
    /// Slice equality: lengths, then `==` on each pair until one fails.
    fn eq(&self, other: &SmallVec<B>) -> bool {
        if self.items.len() != other.items.len() {
            return false;
        }
        let mut i = 0;
        while i < self.items.len() {
            if !(self.items[i] == other.items[i]) {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl<A: Array> Eq for SmallVec<A> where A::Item: Eq {}

impl<A: Array> IntoIterator for SmallVec<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> IntoIter<A> {
        IntoIter { inner: self.items.into_iter() }
    }
}

impl<'a, A: Array> IntoIterator for &'a mut SmallVec<A> {
    type Item = &'a mut A::Item;
    type IntoIter = core::slice::IterMut<'a, A::Item>;

    fn into_iter(self) -> core::slice::IterMut<'a, A::Item> {
        self.items.iter_mut()
    }
}

impl<'a, A: Array> From<&'a [A::Item]> for SmallVec<A>
where
    A::Item: Clone,
{
    fn from(slice: &'a [A::Item]) -> SmallVec<A> {
        let mut v = SmallVec::new();
        v.extend_cloned(slice);
        v
    }
}

impl<A: Array> core::hash::Hash for SmallVec<A>
where
    A::Item: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.items.as_slice().hash(state)
    }
}

impl<A: Array> PartialOrd for SmallVec<A>
where
    A::Item: PartialOrd,
{
    fn partial_cmp(&self, other: &SmallVec<A>) -> Option<core::cmp::Ordering> {
        self.items.as_slice().partial_cmp(other.items.as_slice())
    }
}

impl<A: Array> Ord for SmallVec<A>
where
    A::Item: Ord,
{
    fn cmp(&self, other: &SmallVec<A>) -> core::cmp::Ordering {
        self.items.as_slice().cmp(other.items.as_slice())
    }
}

impl<'a, A: Array> IntoIterator for &'a SmallVec<A> {
    type Item = &'a A::Item;
    type IntoIter = core::slice::Iter<'a, A::Item>;

    fn into_iter(self) -> core::slice::Iter<'a, A::Item> {
        self.items.iter()
    }
}

/// The items by value, front to back; the rest drop with the iterator.
pub struct IntoIter<A: Array> {
    inner: std::vec::IntoIter<A::Item>,
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<A::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<A: Array> DoubleEndedIterator for IntoIter<A> {
    fn next_back(&mut self) -> Option<A::Item> {
        self.inner.next_back()
    }
}

impl<A: Array> ExactSizeIterator for IntoIter<A> {}

/// The items `SmallVec::drain` removes, by value. Upstream writes
/// `T: 'a + Array`; lust has no named-lifetime bounds yet (L0203), and
/// for an array type `T: 'a` holds exactly when its items outlive `'a`,
/// which the `vec::Drain<'a, T::Item>` field already implies.
/// `FusedIterator` is left out until lust has the trait, as for
/// `IntoIter` (a bound on it rejects).
pub struct Drain<'a, T: Array> {
    inner: std::vec::Drain<'a, T::Item>,
}

impl<'a, T: Array> core::fmt::Debug for Drain<'a, T>
where
    T::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Drain").field(&self.inner.as_slice()).finish()
    }
}

impl<'a, T: Array> Iterator for Drain<'a, T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T: Array> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T::Item> {
        self.inner.next_back()
    }
}

impl<'a, T: Array> ExactSizeIterator for Drain<'a, T> {}

/// `smallvec![]`, `smallvec![x; n]`, and `smallvec![a, b, ...]`, as
/// upstream expands them.
macro_rules! smallvec_m {
    (@one $x:expr) => (1usize);
    () => ($crate::smallvec::SmallVec::new());
    ($elem:expr; $n:expr) => ({
        $crate::smallvec::SmallVec::from_elem($elem, $n)
    });
    ($($x:expr),+$(,)?) => ({
        let count = 0usize $(+ $crate::smallvec::smallvec!(@one $x))+;
        let mut vec = $crate::smallvec::SmallVec::new();
        if count <= vec.inline_size() {
            $(vec.push($x);)*
            vec
        } else {
            $crate::smallvec::SmallVec::from_vec(vec![$($x,)+])
        }
    });
}

pub(crate) use smallvec_m as smallvec;
