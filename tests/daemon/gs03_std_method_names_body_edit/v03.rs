use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f() -> u32 {
    let a = 3u32;
    let e: std::sync::mpsc::TrySendError<u32> = std::sync::mpsc::TrySendError::Full(a);
    match e {
        std::sync::mpsc::TrySendError::Full(v) => v,
        _ => 0,
    }
}

fn main() {
    println!("{}", f());
}
