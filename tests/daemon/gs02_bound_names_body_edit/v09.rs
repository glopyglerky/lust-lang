use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    match Excluded(a) {
        Excluded(x) => x + 1,
        _ => 0,
    }
}

fn main() {
    println!("{}", f());
}
