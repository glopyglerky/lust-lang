// expect: L0303
// rustc E0502: the closure reads the Vec that resize_with borrows mutably.
fn main() { let mut v = vec![1, 2]; v.resize_with(4, || v.len()); println!("{v:?}"); }
