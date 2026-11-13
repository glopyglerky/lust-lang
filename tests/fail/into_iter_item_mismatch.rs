// expect: L0110
// rustc E0271: Vec<u16> yields u16, the bound asks for u8.
fn sum<I: IntoIterator<Item = u8>>(i: I) -> u8 { let mut s = 0; for x in i { s += x; } s }
fn main() { println!("{}", sum(vec![1u16, 2])); }
