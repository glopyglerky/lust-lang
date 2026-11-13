use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f(_k: u8) -> u32 {
    let a = 3u32;
    let r = 1..a;
    let s = r.start_bound();
    let _ = s;
    a
}

fn main() {
    println!("{}", f(0));
}
