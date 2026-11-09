// `collect()` into `BTreeMap` / `BTreeSet` (std source): the call is the
// impl's `FromIterator::from_iter`, with `_` holes in a turbofish
// inferred from the items, and drop order as rustc (W-STD-C).
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct D(u8);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn uniq<T: Ord + Clone>(xs: &[T]) -> Vec<T> { let s: BTreeSet<T> = xs.iter().cloned().collect(); s.into_iter().collect() }
fn main() {
    let n = vec![(1, 2)].into_iter().collect::<std::collections::BTreeMap<_, _>>();
    println!("{:?}", n);
    let v = vec![3, 1, 2, 3, 1];
    let s: BTreeSet<i32> = v.iter().copied().collect();
    println!("{:?}", s);
    let m: BTreeMap<&str, usize> = vec!["b", "a", "b"].into_iter().map(|w| (w, w.len())).collect();
    println!("{:?}", m);
    let m2 = v.iter().enumerate().map(|(i, x)| (*x, i)).collect::<BTreeMap<_, _>>();
    println!("{:?}", m2);
    println!("{:?}", uniq(&["z", "a", "z"]));
    let ds: BTreeSet<D> = vec![D(2), D(1), D(2)].into_iter().collect();
    println!("{:?}", ds);
    let w: BTreeSet<char> = "hello".chars().collect();
    println!("{:?}", w);
}
