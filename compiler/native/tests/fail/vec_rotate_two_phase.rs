// expect: L0303
// rustc E0502: rotate_left reaches the slice through DerefMut, so its autoref is no two-phase borrow.
fn main() { let mut v = vec![1, 2, 3]; v.rotate_left(v.len() - 1); println!("{v:?}"); }
