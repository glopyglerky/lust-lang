use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let mut m = std::mem::MaybeUninit::<u32>::uninit();
    *m.write(a)
}

fn main() {
    println!("{}", f());
}
