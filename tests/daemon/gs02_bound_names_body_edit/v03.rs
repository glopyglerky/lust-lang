use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let b: std::ops::Bound<u32> = Unbounded;
    if let Unbounded = b { a } else { 0 }
}

fn main() {
    println!("{}", f());
}
