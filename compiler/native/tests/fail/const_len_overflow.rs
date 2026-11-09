// expect: L0110
// rustc E0080: 200_u8 + 100_u8 overflows in an array length.
const fn add(a: u8, b: u8) -> u8 { a + b }
fn main() { let a = [0u8; add(200, 100) as usize % 7]; println!("{}", a.len()); }
