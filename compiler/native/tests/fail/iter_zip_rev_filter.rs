// expect: L0612
// rustc E0277: Filter is no ExactSizeIterator.
fn main() { let v: Vec<_> = [1, 2].iter().filter(|_| true).zip([1, 2]).rev().collect(); println!("{:?}", v); }
