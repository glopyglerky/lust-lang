// The int mirrors of the float `Eq`/`Ord`/`Hash` reject cases:
// every bound holds, at every nesting shape, and the values print.
use std::hash::{Hash, Hasher};

fn e<T: Eq>(_: &T) -> bool { true }
fn o<T: Ord>(a: &T, b: &T) -> bool { a.cmp(b) == std::cmp::Ordering::Equal }

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct W<T> { f: T }

fn main() {
    println!("{}", e(&(1u8, 2u8)));
    println!("{}", e(&[1u8, 2u8]));
    let v = vec![1u8, 2u8];
    println!("{}", e(&&v[..]));
    println!("{}", e(&(vec![(1u8, 2u8)], 3u8)));
    println!("{}", o(&Box::new((1u64, 2u8)), &Box::new((1u64, 2u8))));
    println!("{}", e(&W { f: (1u8, 2u8) }));
    let mut h = std::collections::hash_map::DefaultHasher::new();
    [1u8, 2u8].hash(&mut h);
    println!("{}", h.finish());
}
