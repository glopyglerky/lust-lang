// expect: L0110
// rustc E0271: &Vec<u8> yields &u8, the bound asks for u8.
fn sum<I: IntoIterator<Item = u8>>(i: I) -> u8 { let mut s = 0; for x in i { s += x; } s }
fn main() { let v = vec![1u8, 2]; println!("{}", sum(&v)); }
