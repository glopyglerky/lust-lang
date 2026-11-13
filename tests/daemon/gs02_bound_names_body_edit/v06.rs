use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f(_k: u8) -> u32 {
    let a = 3u32;
    match Included(a) {
        Included(x) => x,
        _ => 0,
    }
}

fn main() {
    println!("{}", f(0));
}
