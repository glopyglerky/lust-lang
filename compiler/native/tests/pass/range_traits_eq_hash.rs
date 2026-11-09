// The range types' derived PartialEq, Eq, and Hash (core), in derives,
// operators, generic bounds, and hash collections: ts-rust's
// `SourceText` derives Eq over an `Arc<[Range<usize>]>` field.
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq)]
struct SourceText {
    bytes: Arc<[u8]>,
    invalid: Arc<[Range<usize>]>,
}

#[derive(PartialEq, Eq, Hash)]
struct Key {
    r: std::ops::RangeInclusive<u32>,
    to: std::ops::RangeTo<i64>,
}

fn h<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    let a = SourceText { bytes: Arc::from(vec![1u8]), invalid: Arc::from(vec![0..1usize, 3..4]) };
    let b = a.clone();
    let c = SourceText { bytes: Arc::from(vec![1u8]), invalid: Arc::from(vec![0..2usize]) };
    println!("{} {}", a == b, a == c);
    println!("{} {} {} {}", (0..3) == (0..3), (1..=2) == (1..=2), (..5) == (..5), (2..) != (3..));
    let mut r = 1..=1;
    r.next();
    println!("{} {}", r == (1..=1), h(&(0..3u8)) == h(&(0u8, 3u8)));
    println!("{} {} {}", h(&(0..3u32)), h(&(1..=4i64)), h(&(..=7u16)));
    let k = Key { r: 1..=2, to: ..9 };
    println!("{}", h(&k));
    let mut set: HashSet<Range<i32>> = HashSet::new();
    set.insert(0..1);
    set.insert(0..1);
    set.insert(2..3);
    println!("{} {}", set.len(), set.contains(&(2..3)));
    let v: Vec<Range<i32>> = vec![0..1, 2..3];
    println!("{} {}", v.contains(&(2..3)), (.. ) == (..));
}
