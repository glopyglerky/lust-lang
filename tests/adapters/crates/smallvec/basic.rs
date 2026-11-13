// smallvec 1.15.1: the inline-to-heap spill, capacity growth, slice
// methods through Deref, and the smallvec! forms, as ts-rust uses
// SmallVec<[TypeId; N]>.
use smallvec::{SmallVec, smallvec};

fn main() {
    let mut v: SmallVec<[u32; 4]> = SmallVec::new();
    for i in 0..10u32 {
        v.push(i * 3 % 7);
        println!("{} {} {}", v.len(), v.capacity(), v.spilled());
    }
    println!("{:?} {} {:?} {:?}", v, v.contains(&6), v.first(), v.iter().rev().take(2).collect::<Vec<_>>());
    v.sort_unstable();
    v.dedup();
    println!("{:?} {} {}", v, v.inline_size(), v.capacity());
    v.truncate(2);
    v.shrink_to_fit();
    println!("{:?} {} {}", v, v.capacity(), v.spilled());
    let w: SmallVec<[u8; 8]> = smallvec![1, 2, 3];
    let x: SmallVec<[u8; 2]> = smallvec![7; 5];
    println!("{:?} {} {:?} {}", w, w.capacity(), x, x.capacity());
    let y: SmallVec<[u16; 3]> = (0..7u16).collect();
    println!("{:?} {} {:?}", y, y.capacity(), y.clone().into_vec().capacity());
    // drain: the range leaves, the tail moves down, capacity stays.
    let mut d: SmallVec<[u32; 4]> = SmallVec::from_slice(&[1, 2, 3, 4, 5, 6]);
    let mid: Vec<u32> = d.drain(1..4).collect();
    println!("{:?} {:?} {} {}", mid, d, d.len(), d.capacity());
    let mut e: SmallVec<[u32; 4]> = SmallVec::from_slice(&[7, 8, 9]);
    {
        let mut it = e.drain(..);
        println!("{:?} {:?} {:?}", it.next_back(), it, it.size_hint());
    }
    println!("{:?} {} {}", e, e.capacity(), e.spilled());
}
