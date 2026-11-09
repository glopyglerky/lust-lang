// expect: L0612
// rustc E0277: the bound does not give `DoubleEndedIterator`.
fn f(it: impl Iterator<Item = i32>) -> Vec<i32> { it.rev().collect() }
fn main() { println!("{:?}", f(0..3)); }
