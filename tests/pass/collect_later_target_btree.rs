// `collect` into a BTreeMap or BTreeSet whose target only a later use fixes.
use std::collections::{BTreeMap, BTreeSet};

struct W {
    m: BTreeMap<u8, char>,
}

fn mk(src: &[u8]) -> (BTreeSet<u8>, usize) {
    let c = src.iter().copied().collect();
    (c, src.len())
}

fn main() {
    let src = vec![3u8, 1, 2, 1];
    let w = W { m: Some(&src).map(|s| s.iter().map(|&k| (k, (b'a' + k) as char)).collect()).unwrap_or_default() };
    println!("{:?}", w.m);
    let (s, n) = mk(&src);
    println!("{:?} {}", s, n);
    let c = src.iter().rev().copied().collect();
    let t: BTreeSet<u8> = c;
    println!("{:?}", t);
}
