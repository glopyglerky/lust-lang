// expect: L0110
// rustc: the constant `N` is not of type `usize` (an array length).
fn f<const N: u8>(a: [u8; N]) -> usize { a.len() }
fn main() { println!("{}", f([1u8, 2, 3])); }
