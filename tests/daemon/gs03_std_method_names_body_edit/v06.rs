use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

fn f(_k: u8) -> u32 {
    let a = 3u32;
    let (tx, rx) = std::sync::mpsc::sync_channel::<u32>(1);
    let ok = tx.try_send(a).is_ok() as u32;
    ok + rx.recv().unwrap()
}

fn main() {
    println!("{}", f(0));
}
