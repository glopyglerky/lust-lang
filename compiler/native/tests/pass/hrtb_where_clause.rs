fn count<T>(t: &T) -> usize where for<'a> &'a T: IntoIterator { t.into_iter().count() }
fn f<T>(x: T) -> usize where for<'a> &'a T: IntoIterator { (&x).into_iter().count() }
fn main() { println!("{} {}", count(&vec![1, 2]), f(vec![1u8, 2])); }
