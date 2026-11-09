use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let l = std::sync::RwLock::new(a);
    let v = *l.write().unwrap();
    v
}

fn main() {
    println!("{}", f());
}
