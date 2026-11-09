use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let r = 1..a;
    let s = r.end_bound();
    let _ = s;
    a + 2
}

fn main() {
    println!("{}", f());
}
