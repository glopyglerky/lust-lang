// rustc lays Vec and String out as {cap, ptr, len}: the capacity niche is
// at offset 0, so a smaller variant fits after it (generated layout sweep).
#![allow(dead_code)]
use std::mem::size_of;
enum A { V0(u64, u64, u64, u64), V1(u64, Vec<u8>, u32, u32) }
enum B { V0(&'static [u32], Box<u8>, u64), V1(Vec<u8>, char, [u32; 3]) }
enum C { V0(u64, u64, u64, u64), V1(String, char, [u32; 3]), V2(u8) }
enum D { V0(u64, u64, u64), V1(u32, Vec<u16>, u32) }
const Q: [usize; 8] = [
    size_of::<A>(), size_of::<B>(), size_of::<C>(), size_of::<D>(),
    size_of::<Option<A>>(), size_of::<Option<B>>(), size_of::<Option<(u8, Vec<u8>)>>(), size_of::<Option<Option<String>>>(),
];
fn main() {
    println!("{:?}", Q);
}
