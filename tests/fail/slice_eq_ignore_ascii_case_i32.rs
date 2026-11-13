// expect: L0104
// rustc E0599: eq_ignore_ascii_case is a [u8] method.
fn main() { let v = vec![1i32, 2]; let b: bool = v.eq_ignore_ascii_case(&[1, 2]); println!("{b}"); }
