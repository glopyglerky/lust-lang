// The shared hash iterators are Clone: a clone continues from the same
// point, and both walk the same order.
use std::collections::{HashMap, HashSet};
fn twice<I: Iterator + Clone>(i: I) -> usize {
    i.clone().count() + i.count()
}
fn main() {
    let m: HashMap<u8, u16> = (0..20).map(|i| (i, i as u16 * 3)).collect();
    let mut it = m.iter();
    it.next();
    let c = it.clone();
    let a: Vec<_> = it.collect();
    let b: Vec<_> = c.collect();
    println!("{} {}", a == b, a.len());
    let ks = m.keys();
    let s: u32 = ks.clone().map(|&k| k as u32).sum();
    println!("{} {}", s, ks.len());
    let vs = m.values().clone();
    println!("{}", vs.max().unwrap());
    let set: HashSet<u8> = (0..5).collect();
    println!("{}", twice(set.iter()));
}
