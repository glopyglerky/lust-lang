// expect: L0110
// ?Sized param taken by value; rustc E0277. Pin from the opus-fixes review (p56).
fn f<T: ?Sized>(t: T) -> u8 { 1 }
fn main() { println!("{}", f(3u8)); }
