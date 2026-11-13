// expect: L0110
// rustc E0271: the argument's items are `u8`, the bound says `i32`.
fn f(it: impl Iterator<Item = i32>) -> i32 { it.sum() }
fn main() { println!("{}", f(vec![1u8].into_iter())); }
