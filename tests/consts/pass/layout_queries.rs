// ARCH 2.13: layout queries in const position, from the 4.4
// layout engine. offset_of! here is on a repr(C) type only.
use std::mem::{align_of, offset_of, size_of};
#[repr(C)]
struct Header {
    flags: u32,
    object_flags: u32,
    id: u32,
    symbol: u32,
    alias: u64,
}
struct Pair {
    a: u8,
    b: u64,
}
const PAIR_SIZE: usize = size_of::<Pair>();
const PAIR_ALIGN: usize = align_of::<Pair>();
const DATA_OFF: usize = offset_of!(Header, alias);
const _: () = assert!(size_of::<Option<bool>>() == 1);
const _: () = assert!(size_of::<Pair>() == 16);
const ARR: [u8; size_of::<u32>()] = [1, 2, 3, 4];
fn main() {
    println!("{PAIR_SIZE} {PAIR_ALIGN} {DATA_OFF} {} {}", ARR.len(), ARR[3]);
}
