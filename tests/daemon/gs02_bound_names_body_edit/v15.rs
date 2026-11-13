use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    RangeBounds::contains(&(0..5u32), &a) as u32
}

fn main() {
    println!("{}", f());
}
