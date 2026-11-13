// An iterator type whose Iterator impl is generic (`impl<T> Iterator for
// Gen<T>`, std's BTreeSet and VecDeque iterators) through `into_iter` and
// an `I: IntoIterator` bound. The `into_iter` trial left an unbound `?T:
// Sized` obligation from the impl in the queue (L0610 at 1:1).
use std::collections::{BTreeMap, BTreeSet, VecDeque};
struct Gen<T> {
    v: Vec<T>,
}
impl<T> Iterator for Gen<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.v.pop()
    }
}
fn first<I: IntoIterator>(a: I) -> Option<I::Item> {
    a.into_iter().next()
}
fn count<I: IntoIterator>(a: I) -> usize {
    let mut n = 0;
    for _ in a {
        n += 1;
    }
    n
}
fn direct(a: Gen<u8>) -> Option<u8> {
    a.into_iter().next()
}
fn main() {
    println!("{:?}", direct(Gen { v: vec![1u8, 2] }));
    println!("{:?}", first(Gen { v: vec![String::from("a")] }));
    println!("{}", count(Gen { v: vec![1i32, 2, 3] }));
    println!("{:?}", first(BTreeMap::from([(2u8, 'b'), (1, 'a')]).into_keys()));
    println!("{}", count(BTreeSet::from([3u32, 1, 2]).into_iter()));
    println!("{:?}", first(VecDeque::from([7i8, 8]).into_iter()));
}
