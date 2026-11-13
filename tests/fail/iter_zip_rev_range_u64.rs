// expect: L0612
// rustc E0277: Range<u64> is no ExactSizeIterator.
fn main() { let v: Vec<_> = (0..3u64).zip([1, 2]).rev().collect(); println!("{:?}", v); }
