// A bare const item name as a method turbofish argument for a const
// parameter (`chunks.entry::<SLOT_CHUNK, _>(..)`), as a fn path takes it
// (ts-rust ast/synthetic.rs SyntheticArena::slot).
use std::mem::size_of;
struct Slot {
    a: u64,
    b: u32,
}
const CHUNK_BYTES: usize = 1024;
const SLOT_CHUNK: usize = CHUNK_BYTES / size_of::<Slot>();
const PLAIN: usize = 8;
const FLAG: bool = true;
const NEG: i32 = -5;
struct T {
    v: Vec<u32>,
}
impl T {
    fn entry<const PER: usize, E: ?Sized>(&self, index: usize, _e: &E) -> usize {
        index / PER
    }
    fn at<const PER: usize>(&self, index: usize) -> usize {
        index % PER
    }
    fn flag<const F: bool>(&self) -> bool {
        F
    }
    fn neg<const V: i32>(&self) -> i32 {
        V * 2
    }
}
struct S<const N: usize>;
impl<const N: usize> S<N> {
    fn m<const K: usize>(&self) -> usize {
        K + N
    }
}
fn main() {
    let t = T { v: Vec::new() };
    println!("{}", t.entry::<SLOT_CHUNK, u8>(1000, &1));
    println!("{}", t.entry::<PLAIN, _>(1000, "x") + t.v.len());
    println!("{} {}", t.at::<PLAIN>(1001), t.at::<SLOT_CHUNK>(1001));
    println!("{} {}", t.flag::<FLAG>(), t.neg::<NEG>());
    println!("{}", S::<PLAIN>.m::<SLOT_CHUNK>());
}
