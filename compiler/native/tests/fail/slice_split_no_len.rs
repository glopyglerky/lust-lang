// expect: L0104
// rustc E0599: Split is no ExactSizeIterator.
fn main() { let v = vec![1u8, 0]; println!("{}", v.split(|x| *x == 0).len()); }
