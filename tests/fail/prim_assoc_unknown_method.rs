// expect: L0104
// rustc E0599: no associated function `is_ascii_foo` on `u8`.
fn main() { println!("{}", u8::is_ascii_foo(&b'5')); }
