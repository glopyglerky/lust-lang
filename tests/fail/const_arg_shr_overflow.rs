// expect: L0110
// rustc E0080: shift right by 40 on a u32 const argument.
fn g<const V: u32>() -> u32 { V }
fn main() { println!("{}", g::<{ 2u32 >> 40 }>()); }
