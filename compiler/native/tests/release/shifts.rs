// Release semantics: shift amounts are masked to the bit width, also negative and 128-bit ones.
use std::hint::black_box as bb;

fn main() {
    let n8: u32 = bb(9);
    let n64: u32 = bb(70);
    let neg: i32 = bb(-1);
    let big: u128 = bb(130);
    println!("{} {} {}", bb(1u8) << n8, bb(0x80u8) >> n8, bb(-128i8) >> n8);
    println!("{} {} {}", bb(1u64) << n64, bb(u64::MAX) >> n64, bb(-2i64) >> n64);
    println!("{} {} {}", bb(1u32) << neg, bb(1i16) << neg, bb(-1i32) >> neg);
    println!("{} {} {}", bb(1u128) << bb(129u32), bb(u128::MAX) >> bb(200u8), bb(-1i128) >> big);
    println!("{} {}", bb(3i64) << bb(64u64), bb(5usize) >> bb(128u16));
    let mut x: u16 = bb(1);
    x <<= bb(17u8);
    let mut y: i64 = bb(-256);
    y >>= bb(66i32);
    println!("{} {}", x, y);
}
