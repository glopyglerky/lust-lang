// expect: L0612
// rustc E0277: Range<i64> is not ExactSizeIterator.
fn f<I: ExactSizeIterator>(i: I) -> usize { i.len() }
fn main() { println!("{}", f(0i64..5)); }
