//! Go `slices.SortFunc`, `slices.BinarySearchFunc` (go1.27.1
//! `src/slices/sort.go`, `src/slices/zsortanyfunc.go`) and `sort.Slice`
//! (`src/sort/slice.go`, `src/sort/zsortfunc.go`, `src/sort/sort.go`).
//!
//! Go's unstable sorts are pattern-defeating quicksort. The order of equal
//! elements depends on the exact algorithm, so this is a literal port: the
//! resulting permutation equals Go's for every input and comparator.
//!
//! PORT: Go `int` indexes are `isize` here (some Go loops count down past
//! zero); they are cast to `usize` only to index the slice.
//!
//! Also `stable_sort_by`, Rust's stable `sort_by` with one copy on wasm.

use crate::prelude::*;

#[cfg(target_family = "wasm")]
pub use by_index::{sort_func, sort_slice, sort_stable_func, stable_sort_by};

/// wasm: `sort_func`, `sort_stable_func`, `sort_slice` and `stable_sort_by`
/// with one copy of each algorithm for all element types. Each sorts the
/// indexes of `x` (`u32`) with a `dyn` comparator, then puts `x` in that
/// order. The Go algorithms make the same comparisons, in the same order,
/// and the same swaps on the indexes as on `x`, so the result is the same
/// (for `stable_sort_by`, see there). One copy per element and comparator
/// type was 49 KB of the module (123 copies) for the Go sorts, and 50 KB
/// (15 copies) for std's stable sort.
#[cfg(any(test, target_family = "wasm"))]
mod by_index {
    use super::{LessSwap, bits_len, pdqsort_cmp_func, pdqsort_func, stable_cmp_func};
    use std::cmp::Ordering;

    /// Go `slices.SortFunc` (see the native `sort_func`).
    pub fn sort_func<T, F: FnMut(&T, &T) -> i32>(x: &mut [T], mut cmp: F) {
        let mut order = indexes(x);
        let n = order.len() as isize;
        let mut by_index = |a: &u32, b: &u32| cmp(&x[*a as usize], &x[*b as usize]);
        let mut by_index: &mut dyn FnMut(&u32, &u32) -> i32 = &mut by_index;
        pdqsort_cmp_func(&mut order, 0, n, bits_len(n as usize), &mut by_index);
        put_in_order(x, &mut order);
    }

    /// Go `slices.SortStableFunc` (see the native `sort_stable_func`).
    pub fn sort_stable_func<T, F: FnMut(&T, &T) -> i32>(x: &mut [T], mut cmp: F) {
        let mut order = indexes(x);
        let n = order.len() as isize;
        let mut by_index = |a: &u32, b: &u32| cmp(&x[*a as usize], &x[*b as usize]);
        let mut by_index: &mut dyn FnMut(&u32, &u32) -> i32 = &mut by_index;
        stable_cmp_func(&mut order, n, &mut by_index);
        put_in_order(x, &mut order);
    }

    /// Go `sort.Slice` (see the native `sort_slice`).
    pub fn sort_slice<T, F: FnMut(&T, &T) -> bool>(x: &mut [T], mut less: F) {
        let mut order = indexes(x);
        let n = order.len() as isize;
        let mut by_index = |a: &u32, b: &u32| less(&x[*a as usize], &x[*b as usize]);
        let by_index: &mut dyn FnMut(&u32, &u32) -> bool = &mut by_index;
        let mut data = LessSwap {
            data: &mut order,
            less_fn: by_index,
        };
        pdqsort_func(&mut data, 0, n, bits_len(n as usize));
        put_in_order(x, &mut order);
    }

    /// Rust's `x.sort_by(cmp)` (see the native `stable_sort_by`). std's sort
    /// makes other comparisons on the indexes than on `x`. But with a strict
    /// weak order, a stable sort has one correct result, so it is the same.
    pub fn stable_sort_by<T, F: FnMut(&T, &T) -> Ordering>(x: &mut [T], mut cmp: F) {
        let mut order = indexes(x);
        sort_indexes(&mut order, &mut |a, b| {
            cmp(&x[*a as usize], &x[*b as usize])
        });
        put_in_order(x, &mut order);
    }

    /// std's stable sort of `order`, in one function: `sort_by` inlines its
    /// insertion sort for short slices into the caller.
    #[inline(never)]
    fn sort_indexes(order: &mut [u32], cmp: &mut dyn FnMut(&u32, &u32) -> Ordering) {
        order.sort_by(cmp);
    }

    /// The indexes of `x`, in order.
    fn indexes<T>(x: &[T]) -> Vec<u32> {
        (0..u32::try_from(x.len()).expect("a slice of under 4G elements")).collect()
    }

    /// Moves the element at `order[i]` of `x` to `i`, for each `i`, by
    /// following each cycle of `order` with swaps. Leaves `order` as `0..n`.
    fn put_in_order<T>(x: &mut [T], order: &mut [u32]) {
        for start in 0..order.len() {
            let mut current = start;
            loop {
                let next = order[current] as usize;
                order[current] = current as u32;
                if next == start {
                    break;
                }
                x.swap(current, next);
                current = next;
            }
        }
    }
}

/// Rust's stable `x.sort_by(cmp)`, for a `cmp` that is a strict weak order
/// (for example a compare of plain keys). Use it in place of `sort`,
/// `sort_by` and `sort_by_key` in code that the wasm module has: there all
/// element types share one copy of std's sort (`by_index`), and that copy
/// gives the same result only for a strict weak order.
#[cfg(not(target_family = "wasm"))]
pub fn stable_sort_by<T, F: FnMut(&T, &T) -> std::cmp::Ordering>(x: &mut [T], cmp: F) {
    x.sort_by(cmp);
}

// Go: slices/sort.go:30 SortFunc

// Go: slices/sort.go:37 SortStableFunc

// Go: slices/sort.go:152 BinarySearchFunc
/// BinarySearchFunc works like [BinarySearch], but uses a custom comparison
/// function. The slice must be sorted in increasing order, where "increasing"
/// is defined by cmp. cmp should return 0 if the slice element matches
/// the target, a negative number if the slice element precedes the target,
/// or a positive number if the slice element follows the target.
/// cmp must implement the same ordering as the slice, such that if
/// cmp(a, t) < 0 and cmp(b, t) >= 0, then a must precede b in the slice.
pub fn binary_search_func<E, T, F: FnMut(&E, &T) -> i32>(
    x: &[E],
    target: T,
    mut cmp: F,
) -> (usize, bool) {
    let n = x.len();
    // Define cmp(x[-1], target) < 0 and cmp(x[n], target) >= 0 .
    // Invariant: cmp(x[i - 1], target) < 0, cmp(x[j], target) >= 0.
    let (mut i, mut j) = (0usize, n);
    while i < j {
        let h = (i + j) >> 1; // avoid overflow when computing h
        // i ≤ h < j
        if cmp(&x[h], &target) < 0 {
            i = h + 1; // preserves cmp(x[i - 1], target) < 0
        } else {
            j = h; // preserves cmp(x[j], target) >= 0
        }
    }
    // cmp(x[i-1], target) < 0, cmp(x[j], target) >= 0 and j = i => answer is i.
    (i, i < n && cmp(&x[i], &target) == 0)
}

// Go: slices/sort.go:170 sortedHint
/// sortedHint is a hint for pdqsort when choosing the pivot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortedHint {
    UnknownHint,
    IncreasingHint,
    DecreasingHint,
}

// Go: slices/sort.go:179 xorshift
/// xorshift paper: <https://www.jstatsoft.org/article/view/v008i14/xorshift.pdf>
#[derive(Clone, Copy, Debug)]
pub struct Xorshift(pub u64);

impl Xorshift {
    // Go: slices/sort.go:181 Next
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

// Go: slices/sort.go:188 nextPowerOfTwo
pub fn next_power_of_two(length: isize) -> usize {
    1usize << bits_len(length as usize)
}

// PORT: Go `bits.Len(uint(n))`: the minimum number of bits to represent n.
fn bits_len(n: usize) -> isize {
    (usize::BITS - n.leading_zeros()) as isize
}

// ---------------------------------------------------------------------------
// slices/zsortanyfunc.go (generated from sort/gen_sort_variants.go)
// ---------------------------------------------------------------------------

// Go: slices/zsortanyfunc.go:10 insertionSortCmpFunc

// Go: slices/zsortanyfunc.go:20 siftDownCmpFunc

// Go: slices/zsortanyfunc.go:38 heapSortCmpFunc

// Go: slices/zsortanyfunc.go:61 pdqsortCmpFunc

// Go: slices/zsortanyfunc.go:135 partitionCmpFunc

// Go: slices/zsortanyfunc.go:173 partitionEqualCmpFunc

// Go: slices/zsortanyfunc.go:195 partialInsertionSortCmpFunc

// Go: slices/zsortanyfunc.go:240 breakPatternsCmpFunc

// Go: slices/zsortanyfunc.go:261 choosePivotCmpFunc
/// choosePivotCmpFunc chooses a pivot in data[a:b].
///
/// [0,8): chooses a static pivot.
/// [8,shortestNinther): uses the simple median-of-three method.
/// [shortestNinther,∞): uses the Tukey ninther method.
pub fn choose_pivot_cmp_func<T, F: FnMut(&T, &T) -> i32>(
    data: &mut [T],
    a: isize,
    b: isize,
    cmp: &mut F,
) -> (isize, SortedHint) {
    const SHORTEST_NINTHER: isize = 50;
    const MAX_SWAPS: isize = 4 * 3;

    let l = b - a;

    let mut swaps: isize = 0;
    let mut i = a + l / 4 * 1;
    let mut j = a + l / 4 * 2;
    let mut k = a + l / 4 * 3;

    if l >= 8 {
        if l >= SHORTEST_NINTHER {
            // Tukey ninther method, the idea came from Rust's implementation.
            i = median_adjacent_cmp_func(data, i, &mut swaps, cmp);
            j = median_adjacent_cmp_func(data, j, &mut swaps, cmp);
            k = median_adjacent_cmp_func(data, k, &mut swaps, cmp);
        }
        // Find the median among i, j, k and stores it into j.
        j = median_cmp_func(data, i, j, k, &mut swaps, cmp);
    }

    match swaps {
        0 => (j, SortedHint::IncreasingHint),
        MAX_SWAPS => (j, SortedHint::DecreasingHint),
        _ => (j, SortedHint::UnknownHint),
    }
}

// Go: slices/zsortanyfunc.go:298 order2CmpFunc
/// order2CmpFunc returns x,y where data[x] <= data[y], where x,y=a,b or x,y=b,a.
pub fn order2_cmp_func<T, F: FnMut(&T, &T) -> i32>(
    data: &mut [T],
    a: isize,
    b: isize,
    swaps: &mut isize,
    cmp: &mut F,
) -> (isize, isize) {
    if cmp(&data[b as usize], &data[a as usize]) < 0 {
        *swaps += 1;
        return (b, a);
    }
    (a, b)
}

// Go: slices/zsortanyfunc.go:307 medianCmpFunc
/// medianCmpFunc returns x where data[x] is the median of data[a],data[b],data[c], where x is a, b, or c.
pub fn median_cmp_func<T, F: FnMut(&T, &T) -> i32>(
    data: &mut [T],
    a: isize,
    b: isize,
    c: isize,
    swaps: &mut isize,
    cmp: &mut F,
) -> isize {
    let (a, b) = order2_cmp_func(data, a, b, swaps, cmp);
    let (b, c) = order2_cmp_func(data, b, c, swaps, cmp);
    let _ = c;
    let (_a, b) = order2_cmp_func(data, a, b, swaps, cmp);
    b
}

// Go: slices/zsortanyfunc.go:315 medianAdjacentCmpFunc
/// medianAdjacentCmpFunc finds the median of data[a - 1], data[a], data[a + 1] and stores the index into a.
pub fn median_adjacent_cmp_func<T, F: FnMut(&T, &T) -> i32>(
    data: &mut [T],
    a: isize,
    swaps: &mut isize,
    cmp: &mut F,
) -> isize {
    median_cmp_func(data, a - 1, a, a + 1, swaps, cmp)
}

// Go: slices/zsortanyfunc.go:319 reverseRangeCmpFunc

// Go: slices/zsortanyfunc.go:329 swapRangeCmpFunc

// Go: slices/zsortanyfunc.go:335 stableCmpFunc

// Go: slices/zsortanyfunc.go:378 symMergeCmpFunc

// Go: slices/zsortanyfunc.go:464 rotateCmpFunc

// ---------------------------------------------------------------------------
// sort.Slice: sort/slice.go and sort/zsortfunc.go (generated from the same
// template as slices/zsortanyfunc.go, over a lessSwap instead of cmp)
// ---------------------------------------------------------------------------

// Go: sort/slice.go:24 Slice

// Go: sort/sort.go:85 lessSwap
/// lessSwap is a pair of Less and Swap function for use with the
/// auto-generated func-optimized variant of sort.go in
/// zfuncversion.go.
pub struct LessSwap<'a, T, F: FnMut(&T, &T) -> bool> {
    pub data: &'a mut [T],
    pub less_fn: F,
}

impl<T, F: FnMut(&T, &T) -> bool> LessSwap<'_, T, F> {
    /// Go `data.Less(i, j)`.
    pub fn less(&mut self, i: isize, j: isize) -> bool {
        (self.less_fn)(&self.data[i as usize], &self.data[j as usize])
    }

}

// Go: sort/zsortfunc.go:10 insertionSort_func

// Go: sort/zsortfunc.go:20 siftDown_func

// Go: sort/zsortfunc.go:38 heapSort_func

// Go: sort/zsortfunc.go:61 pdqsort_func

// Go: sort/zsortfunc.go:135 partition_func

// Go: sort/zsortfunc.go:173 partitionEqual_func

// Go: sort/zsortfunc.go:195 partialInsertionSort_func

// Go: sort/zsortfunc.go:240 breakPatterns_func

// Go: sort/zsortfunc.go:261 choosePivot_func
/// choosePivot_func chooses a pivot in data[a:b].
///
/// [0,8): chooses a static pivot.
/// [8,shortestNinther): uses the simple median-of-three method.
/// [shortestNinther,∞): uses the Tukey ninther method.
pub fn choose_pivot_func<T, F: FnMut(&T, &T) -> bool>(
    data: &mut LessSwap<'_, T, F>,
    a: isize,
    b: isize,
) -> (isize, SortedHint) {
    const SHORTEST_NINTHER: isize = 50;
    const MAX_SWAPS: isize = 4 * 3;

    let l = b - a;

    let mut swaps: isize = 0;
    let mut i = a + l / 4 * 1;
    let mut j = a + l / 4 * 2;
    let mut k = a + l / 4 * 3;

    if l >= 8 {
        if l >= SHORTEST_NINTHER {
            // Tukey ninther method, the idea came from Rust's implementation.
            i = median_adjacent_func(data, i, &mut swaps);
            j = median_adjacent_func(data, j, &mut swaps);
            k = median_adjacent_func(data, k, &mut swaps);
        }
        // Find the median among i, j, k and stores it into j.
        j = median_func(data, i, j, k, &mut swaps);
    }

    match swaps {
        0 => (j, SortedHint::IncreasingHint),
        MAX_SWAPS => (j, SortedHint::DecreasingHint),
        _ => (j, SortedHint::UnknownHint),
    }
}

// Go: sort/zsortfunc.go:298 order2_func
/// order2_func returns x,y where data[x] <= data[y], where x,y=a,b or x,y=b,a.
pub fn order2_func<T, F: FnMut(&T, &T) -> bool>(
    data: &mut LessSwap<'_, T, F>,
    a: isize,
    b: isize,
    swaps: &mut isize,
) -> (isize, isize) {
    if data.less(b, a) {
        *swaps += 1;
        return (b, a);
    }
    (a, b)
}

// Go: sort/zsortfunc.go:307 median_func
/// median_func returns x where data[x] is the median of data[a],data[b],data[c], where x is a, b, or c.
pub fn median_func<T, F: FnMut(&T, &T) -> bool>(
    data: &mut LessSwap<'_, T, F>,
    a: isize,
    b: isize,
    c: isize,
    swaps: &mut isize,
) -> isize {
    let (a, b) = order2_func(data, a, b, swaps);
    let (b, c) = order2_func(data, b, c, swaps);
    let _ = c;
    let (_a, b) = order2_func(data, a, b, swaps);
    b
}

// Go: sort/zsortfunc.go:315 medianAdjacent_func
/// medianAdjacent_func finds the median of data[a - 1], data[a], data[a + 1] and stores the index into a.
pub fn median_adjacent_func<T, F: FnMut(&T, &T) -> bool>(
    data: &mut LessSwap<'_, T, F>,
    a: isize,
    swaps: &mut isize,
) -> isize {
    median_func(data, a - 1, a, a + 1, swaps)
}

// Go: sort/zsortfunc.go:319 reverseRange_func

// Go: sort/zsortfunc.go:329 swapRange_func

// Go: sort/zsortfunc.go:335 stable_func

// Go: sort/zsortfunc.go:378 symMerge_func

// Go: sort/zsortfunc.go:464 rotate_func

#[cfg(test)]
mod tests {
    use super::*;

    /// A `(file, pos, id)` key and the comparator shape of Go
    /// `compareSymbolsWorker`: files 0 and 1 are in the file index map
    /// (indexes 0 and 1), files 2 and 3 are not and read as index 0. So keys
    /// in files 0, 2 and 3 compare by position in one file and by id across
    /// files, which is not transitive.
    #[derive(Clone, Copy)]
    struct Key {
        file: i32,
        pos: i32,
        id: i32,
    }

    fn file_index(file: i32) -> i32 {
        i32::from(file == 1)
    }

    fn cmp_keys(a: &Key, b: &Key) -> i32 {
        if a.id == b.id {
            return 0;
        }
        if a.file != b.file {
            let r = file_index(a.file) - file_index(b.file);
            if r != 0 {
                return r;
            }
        } else if a.pos != b.pos {
            return a.pos - b.pos;
        }
        a.id - b.id
    }

    fn keys(n: i32) -> Vec<Key> {
        let mut seed: u32 = 12345;
        let mut next = || {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (seed >> 16) as i32
        };
        (0..n)
            .map(|i| {
                let file = next() % 4;
                let pos = next() % 100;
                Key {
                    file,
                    pos,
                    id: n - i,
                }
            })
            .collect()
    }

    fn ids(keys: &[Key]) -> Vec<i32> {
        keys.iter().map(|k| k.id).collect()
    }

    // Rust std `sort_by` and `sort_unstable_by` (1.93.0) panic on these 64
    // keys: "user-provided comparison function does not correctly implement
    // a total order". The expected orders are the output of Go 1.26.8
    // `slices.SortFunc`, `slices.SortStableFunc` and `sort.Slice` on the same
    // keys and comparator.
    #[test]
    fn non_total_order_sorts_as_go() {
        let go_unstable = [
            13, 5, 1, 9, 3, 29, 14, 12, 16, 10, 17, 18, 24, 31, 4, 19, 2, 21, 28, 27, 32, 11, 37,
            26, 20, 41, 7, 35, 44, 39, 34, 46, 60, 47, 45, 51, 48, 57, 58, 36, 59, 62, 49, 54, 53,
            64, 25, 30, 22, 50, 40, 15, 8, 33, 43, 52, 55, 42, 56, 6, 61, 38, 23, 63,
        ];
        let go_stable = [
            13, 5, 1, 9, 3, 14, 12, 16, 10, 4, 2, 17, 18, 32, 11, 20, 7, 24, 35, 28, 27, 36, 37,
            26, 41, 44, 29, 59, 51, 48, 39, 31, 34, 19, 21, 60, 47, 45, 58, 62, 49, 46, 54, 57, 53,
            64, 25, 30, 22, 50, 40, 15, 8, 33, 43, 52, 55, 42, 56, 6, 61, 38, 23, 63,
        ];

        let mut a = keys(64);
        sort_func(&mut a, cmp_keys);
        assert_eq!(ids(&a), go_unstable);

        let mut b = keys(64);
        sort_stable_func(&mut b, cmp_keys);
        assert_eq!(ids(&b), go_stable);

        let mut c = keys(64);
        sort_slice(&mut c, |x, y| cmp_keys(x, y) < 0);
        assert_eq!(ids(&c), go_unstable);
    }

    /// The wasm sorts (`by_index`) give the same permutation as the direct
    /// sorts, for a comparator that is not transitive and for one that
    /// answers at random: they make the same comparisons in the same order.
    #[test]
    fn index_sorts_match_direct_sorts() {
        for n in [0, 1, 2, 12, 13, 20, 21, 50, 64, 100, 300] {
            let input = keys(n);
            let (mut want, mut got) = (input.clone(), input.clone());
            sort_func(&mut want, cmp_keys);
            by_index::sort_func(&mut got, cmp_keys);
            assert_eq!(ids(&got), ids(&want));
            let (mut want, mut got) = (input.clone(), input.clone());
            sort_stable_func(&mut want, cmp_keys);
            by_index::sort_stable_func(&mut got, cmp_keys);
            assert_eq!(ids(&got), ids(&want));
            let (mut want, mut got) = (input.clone(), input.clone());
            sort_slice(&mut want, |x, y| cmp_keys(x, y) < 0);
            by_index::sort_slice(&mut got, |x, y| cmp_keys(x, y) < 0);
            assert_eq!(ids(&got), ids(&want));

            let seed = n as u64 + 7;
            let (mut a, mut b) = (Xorshift(seed), Xorshift(seed));
            let (mut want, mut got) = (input.clone(), input.clone());
            sort_func(&mut want, |_, _| (a.next() % 3) as i32 - 1);
            by_index::sort_func(&mut got, |_, _| (b.next() % 3) as i32 - 1);
            assert_eq!(ids(&got), ids(&want));
            let (mut want, mut got) = (input.clone(), input.clone());
            sort_stable_func(&mut want, |_, _| (a.next() % 3) as i32 - 1);
            by_index::sort_stable_func(&mut got, |_, _| (b.next() % 3) as i32 - 1);
            assert_eq!(ids(&got), ids(&want));
            let (mut want, mut got) = (input.clone(), input);
            sort_slice(&mut want, |_, _| a.next() % 2 == 0);
            by_index::sort_slice(&mut got, |_, _| b.next() % 2 == 0);
            assert_eq!(ids(&got), ids(&want));
        }
    }

    /// The wasm `stable_sort_by` gives the same order as std's `sort_by` for
    /// strict weak orders with many equal keys, at sizes that reach std's
    /// insertion sort and its driftsort.
    #[test]
    fn index_stable_sort_matches_sort_by() {
        for n in [0, 1, 2, 19, 20, 21, 50, 64, 100, 300, 1000, 5000] {
            let input = keys(n);
            let (mut want, mut got) = (input.clone(), input.clone());
            want.sort_by(|a, b| (a.file, a.pos).cmp(&(b.file, b.pos)));
            by_index::stable_sort_by(&mut got, |a, b| (a.file, a.pos).cmp(&(b.file, b.pos)));
            assert_eq!(ids(&got), ids(&want));
            let (mut want, mut got) = (input.clone(), input);
            want.sort_by(|a, b| b.file.cmp(&a.file));
            by_index::stable_sort_by(&mut got, |a, b| b.file.cmp(&a.file));
            assert_eq!(ids(&got), ids(&want));
        }
    }

    /// A comparator with a random answer on every call, at sizes that reach
    /// insertion sort, pdqsort, the heapsort fallback and symMerge. The sorts
    /// must not panic and must return a permutation.
    #[test]
    fn random_comparator_does_not_panic() {
        let mut rng = Xorshift(7);
        for n in [0, 1, 2, 12, 13, 20, 21, 50, 100, 300] {
            let input: Vec<u32> = (0..n).collect();
            let mut a = input.clone();
            sort_func(&mut a, |_, _| (rng.next() % 3) as i32 - 1);
            let mut b = input.clone();
            sort_stable_func(&mut b, |_, _| (rng.next() % 3) as i32 - 1);
            let mut c = input.clone();
            sort_slice(&mut c, |_, _| rng.next() % 2 == 0);
            for mut out in [a, b, c] {
                out.sort_unstable();
                assert_eq!(out, input);
            }
        }
    }
}
