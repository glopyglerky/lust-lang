// expect: L0304
// rustc E0499: a live `Drain` gives its tail back when dropped, so its
// `&mut v` lasts to the end of the scope.
fn main() { let mut v = vec![1, 2]; let d = v.drain(..1); v.push(3); println!("{}", v.len()); }
