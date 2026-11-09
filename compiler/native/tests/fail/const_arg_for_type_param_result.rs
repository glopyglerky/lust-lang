// expect: L0110
// rustc E0747: a braced const in the second type-param slot.
fn main() { let _x: Result<u8, { 3 }> = Ok(1); }
