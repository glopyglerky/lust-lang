// expect: L0110
// rustc E0308: `drop::<u8>` as a value takes a `u8`.
fn main() { let f = drop::<u8>; f(1u16); }
