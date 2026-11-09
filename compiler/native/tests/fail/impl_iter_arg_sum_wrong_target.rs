// expect: L0612
// rustc E0277: `i64: Sum<i32>` does not hold for an `impl Iterator<Item = i32>`.
fn f(it: impl Iterator<Item = i32>) -> i64 { it.sum() }
fn main() { println!("{}", f(0..3)); }
