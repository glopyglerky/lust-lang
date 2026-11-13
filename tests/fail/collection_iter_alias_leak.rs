// expect: L0110
// A collection iterator is its own std type, not the builtin
// iterator it wraps: `VecDeque::iter()` is a `vec_deque::Iter`, not
// a `slice::Iter` (rustc E0308).
use std::collections::VecDeque;
fn count_slice(it: std::slice::Iter<'_, i32>) -> usize {
    it.count()
}
fn main() {
    let d: VecDeque<i32> = VecDeque::from(vec![1, 2]);
    println!("{}", count_slice(d.iter()));
}
