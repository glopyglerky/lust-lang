// expect: L0612
// rustc E0277: RangeInclusive<i32> is no ExactSizeIterator.
fn main() { let v: Vec<_> = (0..=3).zip([1, 2]).rev().collect(); println!("{:?}", v); }
