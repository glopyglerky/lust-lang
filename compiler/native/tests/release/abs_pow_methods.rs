// Release semantics: core methods that inherit the overflow checks wrap (abs, pow,
// next_power_of_two, next_multiple_of); strict and checked forms keep their own behavior.
use std::hint::black_box as bb;

fn main() {
    println!("{} {} {} {}", bb(i8::MIN).abs(), bb(i32::MIN).abs(), bb(i64::MIN).abs(), bb(i128::MIN).abs());
    println!("{} {} {}", bb(3u8).pow(bb(9)), bb(-3i16).pow(bb(11)), bb(7u64).pow(bb(40)));
    println!("{} {}", bb(3u128).pow(bb(100)), bb(-5i128).pow(bb(61)));
    println!("{} {} {}", bb(200u8).next_power_of_two(), bb(u32::MAX).next_power_of_two(), bb(1u64 << 63).next_power_of_two());
    println!("{} {}", bb(250u8).next_multiple_of(bb(7)), bb(u128::MAX - 3).next_multiple_of(bb(8)));
    println!("{:?} {:?} {:?}", bb(200u8).checked_add(bb(100)), bb(i8::MIN).checked_abs(), bb(2u8).checked_pow(bb(8)));
    println!("{} {}", bb(i8::MIN).wrapping_abs(), bb(i32::MIN).unsigned_abs());
}
