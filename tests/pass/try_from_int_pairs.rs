// Scalar `TryFrom` and `TryInto` (fx-tryfrom): checked integer
// conversions at the edge values, the char pairs, integer to
// `bool`, and the `Infallible` widening pairs, in assoc, method,
// qualified, and trait-name forms.
fn main() {
    println!("{:?} {:?}", u8::try_from(255u32), u8::try_from(256u32));
    println!("{:?} {:?}", u8::try_from(0i32), u8::try_from(-1i32));
    println!("{:?} {:?}", i8::try_from(127i32), i8::try_from(128i32));
    println!("{:?} {:?}", i8::try_from(-128i32), i8::try_from(-129i32));
    println!("{:?} {:?}", u32::try_from(u64::MAX), u64::try_from(i64::MIN));
    println!("{:?} {:?}", i128::try_from(u128::MAX), u128::try_from(-1i128));
    println!("{:?} {:?}", usize::try_from(u64::MAX), u32::try_from(usize::MAX));
    println!("{:?} {:?}", isize::try_from(i64::MIN), i16::try_from(isize::MAX));
    println!("{} {}", u8::try_from(300u32).unwrap_err(), u8::try_from(-5i32).unwrap_err());
    println!("{}", u8::try_from(300u32).unwrap_err() == u8::try_from(400u32).unwrap_err());
    println!("{}", u8::try_from(300u32).unwrap_err() == u8::try_from(-5i32).unwrap_err());
    let x: i64 = -5;
    println!("{:?} {}", u8::try_from(x), x.try_into().unwrap_or(9u8));
    let m: Result<u8, std::num::TryFromIntError> = 300i32.try_into();
    println!("{:?}", m);
    let n: u16 = 3u32.try_into().unwrap();
    println!("{}", n);
    println!("{:?}", <u8 as TryFrom<i32>>::try_from(-1));
    println!("{:?}", <i32 as TryInto<u8>>::try_into(300));
    let r: Result<u8, _> = TryFrom::try_from(300i32);
    println!("{:?}", r);
    let inf: Result<u32, std::convert::Infallible> = u32::try_from(3u8);
    println!("{:?}", inf);
    let inf2: Result<i64, std::convert::Infallible> = (-3i32).try_into();
    println!("{:?}", inf2);
    println!("{:?} {:?} {:?}", char::try_from(65u32), char::try_from(0x10FFFFu32), char::try_from(0x110000u32));
    println!("{:?} {}", char::try_from(0xD800u32), char::try_from(0xD800u32).unwrap_err());
    println!("{:?} {:?} {:?}", u8::try_from('a'), u16::try_from('\u{10FFFF}'), usize::try_from('\u{10FFFF}'));
    println!("{:?} {}", u8::try_from('\u{100}'), u8::try_from('\u{100}').unwrap_err());
    println!("{:?} {:?}", char::try_from(65u8), u32::try_from('a'));
    println!("{:?} {:?} {:?} {:?}", bool::try_from(0u8), bool::try_from(1i32), bool::try_from(2u8), bool::try_from(-1i8));
    println!("{:?}", u8::try_from(300));
    println!("{:?}", u8::try_from(5usize));
}
