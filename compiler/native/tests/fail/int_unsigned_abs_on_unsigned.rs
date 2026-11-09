// expect: L0104
// rustc E0599: `unsigned_abs` exists only on signed integers.
fn main() { println!("{}", 5u32.unsigned_abs()); }
