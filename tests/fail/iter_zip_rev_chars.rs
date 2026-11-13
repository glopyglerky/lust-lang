// expect: L0612
// rustc E0277: Chars is no ExactSizeIterator, so the Zip is not double-ended.
fn main() { let v: Vec<_> = "ab".chars().zip([1, 2]).rev().collect(); println!("{:?}", v); }
