// expect: L0110
// rustc E0308: the const item is a u8.
const L: u8 = 2;
fn f(a: [u8; L]) -> usize { a.len() }
fn main() { println!("{}", f([1, 2])); }
