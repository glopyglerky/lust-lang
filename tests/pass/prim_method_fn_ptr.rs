// An integer type's builtin methods coerce to fn pointers (`let f: fn(u32)
// -> u32 = u32::isqrt;`): by value and by reference, `From` conversions,
// tuple and Option returns, casts, and if/match joins of two methods.
fn ap(f: fn(u32, u32) -> u32) -> u32 {
    f(5, 3)
}
fn main() {
    let n = std::env::args().count() as u32;
    let f: fn(u32) -> u32 = u32::isqrt;
    let g: fn(&u8) -> bool = u8::is_ascii_digit;
    let h: fn(u8) -> u64 = u64::from;
    let o: fn(i32, u32) -> (i32, bool) = i32::overflowing_pow;
    let c: fn(u32) -> Option<u32> = u32::checked_next_power_of_two;
    println!("{} {} {} {:?} {:?}", f(17), g(&b'7'), h(9), o(5, 30), c(5));
    println!("{} {}", ap(u32::wrapping_sub), ap(u32::max));
    let k = if n > 1 { u32::count_ones } else { u32::leading_zeros };
    let m = match n { 1 => u32::isqrt, 2 => u32::count_ones, _ => u32::isqrt };
    println!("{} {} {}", k(7), m(9), (i64::abs as fn(i64) -> i64)(-4));
    let w: fn(u128, u128) -> u128 = u128::wrapping_mul;
    println!("{}", w(u128::MAX, 3));
}
