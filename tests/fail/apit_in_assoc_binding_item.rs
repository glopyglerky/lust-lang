// expect: L0612
// rustc E0277: the items are `i32`, and `String: From<i32>` does not hold.
fn args(xs: impl IntoIterator<Item = impl Into<String>>) -> usize { let mut n = 0; for x in xs { let s: String = x.into(); n += s.len(); } n }
fn main() { println!("{}", args(vec![1, 2])); }
