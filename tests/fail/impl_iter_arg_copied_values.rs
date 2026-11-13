// expect: L0612
// rustc E0271: `copied` needs items that are references.
fn f(it: impl Iterator<Item = i32>) -> i32 { it.copied().sum() }
fn main() { println!("{}", f(0..3)); }
