// expect: L0110
// rustc E0080: the same overflow in an array type.
const fn add(a: u8, b: u8) -> u8 { a + b }
fn main() { let a: [u8; add(200, 100) as usize % 7] = [0; 6]; println!("{}", a.len()); }
