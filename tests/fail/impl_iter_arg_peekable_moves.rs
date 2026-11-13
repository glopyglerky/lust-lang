// expect: L0301
// rustc E0382: `peekable` moves the iterator.
fn f(mut it: impl Iterator<Item = i32>) -> Option<i32> { let _p = it.peekable(); it.next() }
fn main() { println!("{:?}", f(0..3)); }
