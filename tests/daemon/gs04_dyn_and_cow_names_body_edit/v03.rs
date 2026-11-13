use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let r: std::thread::Result<u32> = Ok(a);
    r.unwrap_or(0)
}

fn main() {
    println!("{}", f());
}
