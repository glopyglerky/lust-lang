// expect: L0303
// rustc E0502: the replacement iterator borrows the Vec splice borrows mutably.
fn main() { let mut v = vec![1, 2]; v.splice(..1, v.iter().copied()); println!("{v:?}"); }
