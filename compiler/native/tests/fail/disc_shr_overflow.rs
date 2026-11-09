// expect: L0110
// rustc E0080: shift right by 8 on a u8 discriminant.
#[repr(u8)]
enum E { X = 1u8 >> 8 }
fn main() { println!("{}", E::X as u8); }
