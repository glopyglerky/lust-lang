// expect: L0110
// rustc E0308: the const fn returns u8.
const fn f() -> u8 { 3 }
fn main() { let a: [u8; f()] = [1, 2, 3]; println!("{}", a.len()); }
