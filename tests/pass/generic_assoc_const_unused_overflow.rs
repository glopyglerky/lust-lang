// A generic associated const that would overflow for an instance nobody
// uses is no error (rustc evaluates it per used instance only).
trait Tr {
    const C: u8;
    const D: u8 = Self::C + 200;
}
struct S;
struct T2;
impl Tr for S {
    const C: u8 = 100;
}
impl Tr for T2 {
    const C: u8 = 10;
}
fn g<T: Tr>() -> u8 {
    T::D
}
#[allow(dead_code)]
fn h<T: Tr>() -> u8 {
    T::D + g::<T>() / 2
}
fn main() {
    println!("{}", g::<T2>());
}
