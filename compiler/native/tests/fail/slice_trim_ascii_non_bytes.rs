// expect: L0104
// rustc E0599: trim_ascii is a [u8] method only.
fn main() { let v = vec![1i32, 2]; println!("{:?}", v.trim_ascii()); }
