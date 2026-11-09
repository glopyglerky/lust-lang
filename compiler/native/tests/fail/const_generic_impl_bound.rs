// expect: L0104
// rustc E0599: the impl needs T: Copy.
trait Len { fn len2(&self) -> usize; }
impl<T: Copy, const N: usize> Len for [T; N] { fn len2(&self) -> usize { N * 2 } }
fn main() { let a = [String::new()]; println!("{}", a.len2()); }
