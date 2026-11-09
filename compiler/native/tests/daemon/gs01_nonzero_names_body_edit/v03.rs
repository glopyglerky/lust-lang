use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    std::num::NonZero::new(a).unwrap().get()
}

fn main() {
    println!("{}", f());
}
