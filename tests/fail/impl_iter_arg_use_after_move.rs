// expect: L0301
// rustc E0382: `count` takes the iterator by value.
fn f(it: impl Iterator<Item = i32>) -> usize { let n = it.count(); n + it.count() }
fn main() { println!("{}", f(0..3)); }
