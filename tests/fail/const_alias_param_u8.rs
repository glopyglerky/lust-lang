// expect: L0110
// rustc: the constant `N` is not of type `usize` (an array length).
type A<const N: u8> = [u8; N];
fn main() { println!("{}", 1); }
