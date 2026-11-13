use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    std::num::NonZeroU64::new(5).unwrap().get() as u32
}

fn main() {
    println!("{}", f());
}
