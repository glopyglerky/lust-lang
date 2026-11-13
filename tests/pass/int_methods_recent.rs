// Integer methods stabilized in Rust 1.84 to 1.97 (fx-defaults,
// W-BENCH3 bug 10): is_multiple_of, unbounded shifts, the signed
// casts, midpoint, isqrt, the bit-width and isolate forms, the
// strict family, the mixed-sign families, and the carrying forms.
// The const values run through the const evaluator's own rows.
const CM: bool = 12usize.is_multiple_of(3);
const CS: u32 = 17u32.isqrt();
const CMID: i64 = i64::midpoint(-5, -2);
const CCAST: u8 = (-1i8).cast_unsigned();
const CBITS: u32 = 0b1010u32.bit_width();
const CSTRICT: u32 = 3u32.strict_add(4);
const CMIX: Option<u32> = 10u32.checked_sub_signed(-3);
fn main() {
    println!("{} {} {}", 6usize.is_multiple_of(2), 0u32.is_multiple_of(0), 5u64.is_multiple_of(0));
    println!("{} {} {}", 1u32.unbounded_shl(40), (-8i32).unbounded_shr(2), 3u8.unbounded_shl(9));
    println!("{} {}", 255u8.cast_signed(), (-1i16).cast_unsigned());
    println!("{} {} {}", u32::midpoint(0, 10), i32::midpoint(-3, 0), i64::midpoint(i64::MIN, i64::MAX));
    println!("{} {} {:?} {}", 17u32.isqrt(), 16i32.isqrt(), (-4i32).checked_isqrt(), 9i64.checked_isqrt().unwrap());
    println!("{} {:?} {:?} {} {}", 5u32.bit_width(), 5u32.highest_one(), 0u32.lowest_one(), 0b1010u32.isolate_lowest_one(), 0b1010u32.isolate_highest_one());
    println!("{} {} {} {}", 3u32.strict_add(4), 10i64.strict_div_euclid(-3), 2u32.strict_pow(5), 1u32.strict_shl(3));
    println!("{:?} {:?} {} {}", 10u32.checked_sub_signed(-3), 5i32.checked_add_unsigned(3u32), 5u32.saturating_sub_signed(9), 5i32.wrapping_sub_unsigned(9u32));
    println!("{:?} {:?}", 250u8.overflowing_add_signed(10i8), 10u32.checked_signed_diff(4));
    println!("{:?} {:?} {:?}", u64::MAX.carrying_add(1, false), 0u64.borrowing_sub(1, true), 3u64.carrying_mul(4, 5));
    println!("{:?} {}", 3u64.carrying_mul_add(4, 5, 6), 200u8.strict_add_signed(-50i8));
    println!("{} {} {} {} {} {:?} {:?}", CM, CS, CMID, CCAST, CBITS, CSTRICT, CMIX);
    let wide: u128 = (1u128 << 100) + 8;
    println!("{} {} {:?} {}", wide.is_multiple_of(8), wide.bit_width(), wide.highest_one(), wide.cast_signed() > 0);
}
