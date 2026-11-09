// Every derive-family trait on tuples at the exact core limit (12 elements),
// plus the unit tuple and 1-tuples: direct uses, derives over a tuple field,
// and generic bounds proved by a 12-tuple.
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Clone)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}

fn d<T: Default>() -> T {
    T::default()
}
fn e<T: PartialEq>(a: &T) -> bool {
    a == a
}
fn q<T: Eq + Ord>(a: &T, b: &T) -> bool {
    a.cmp(b) == std::cmp::Ordering::Equal
}

fn main() {
    let t = (1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8);
    println!("{:?}", t);
    println!("{} {} {:?}", t == t, t < t, t.cmp(&t));
    let mut h = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut h);
    println!("{}", h.finish());
    let z: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) = d();
    println!("{} {} {}", z.0, z.11, e(&t));
    println!("{}", q(&t, &t));

    let s = S { t };
    println!("{:?}", s);
    println!("{} {:?}", s == s.clone(), s.cmp(&s));
    println!("{:?}", S::default());
    let mut set = std::collections::HashSet::new();
    set.insert(s);
    println!("{}", set.len());

    let u = ();
    println!("{:?} {} {}", u, u == u, u < u);
    let one = (7u8,);
    println!("{:?} {} {:?}", one, one == one, one.cmp(&one));
    let one_d: (u8,) = d();
    println!("{}", one_d.0);
}
