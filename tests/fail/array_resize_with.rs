// expect: L0104
// rustc E0599: resize_with is a Vec method, not a slice or array one.
fn main() { let mut v = [1, 2]; v.resize_with(3, || 0); println!("{v:?}"); }
