// An unsuffixed literal passed for a type param with an `Into`, `From`,
// or shift bound falls back to i32 (or f64); the bound's type argument
// does not name it. A target with one fitting impl names it (`char`
// from u8, `i8` from i8).
use std::ops::{Shl, Shr, ShlAssign};
trait Name {
    fn name(&self) -> &'static str;
}
impl Name for i32 {
    fn name(&self) -> &'static str {
        "i32"
    }
}
impl Name for i64 {
    fn name(&self) -> &'static str {
        "i64"
    }
}
fn named<T: Into<i64> + Name>(t: T) -> &'static str {
    t.name()
}
fn widen<T: Into<i64>>(t: T) -> i64 {
    t.into()
}
fn widen_f<T: Into<f64>>(t: T) -> f64 {
    t.into()
}
fn via_where<T>(t: T) -> u64
where
    u64: From<T>,
{
    u64::from(t)
}
fn shl<T: Shl<i64, Output = T> + std::fmt::Debug>(t: T) -> T {
    t << 4i64
}
fn shr<T: Shr<u32, Output = T>>(t: T) -> T {
    t >> 1u32
}
fn shl_assign<T: ShlAssign<u8>>(mut t: T) -> T {
    t <<= 2u8;
    t
}
fn main() {
    println!("{} {} {}", named(5), widen(5), widen(3u8));
    println!("{} {}", widen_f(5), widen_f(1.5));
    println!("{} {}", via_where(3u8), via_where(7u32));
    println!("{:?} {:?} {} {}", shl(1), shl(3u8), shr(-8), shl_assign(1));
    let a: i64 = From::from(3);
    let b: f64 = From::from(2.5);
    let c: i8 = From::from(3);
    let d: char = From::from(65);
    let e: f32 = From::from(2.5);
    println!("{} {} {} {} {} {}", a, b, c, d, e, i64::from(4));
}
