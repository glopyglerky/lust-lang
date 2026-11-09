// expect: L0612
// rustc E0277: with no `Item` binding, `i32: Sum<I::Item>` is unknown.
fn f(it: impl Iterator) -> i32 { it.sum() }
fn main() { println!("{}", f(0..3)); }
