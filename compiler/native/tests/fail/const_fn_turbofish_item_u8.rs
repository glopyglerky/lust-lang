// expect: L0110
// rustc E0308: the const item is a u8.
const L: u8 = 3;
fn f<const N: usize>() -> usize { N }
fn main() { println!("{}", f::<L>()); }
