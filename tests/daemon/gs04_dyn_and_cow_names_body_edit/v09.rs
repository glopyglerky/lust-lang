use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let c: std::borrow::Cow<str> = std::borrow::Cow::Borrowed("ab");
    c.len() as u32
}

fn main() {
    println!("{}", f());
}
