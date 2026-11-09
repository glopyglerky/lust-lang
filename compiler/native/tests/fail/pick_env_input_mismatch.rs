// expect: L0110
// rustc E0308: the bound fixes the input as u8.
trait Add2<X> { fn add2(&self, x: X) -> u32; }
impl Add2<u8> for u16 { fn add2(&self, x: u8) -> u32 { *self as u32 + x as u32 } }
fn go<U: Add2<u8>>(u: U) -> u32 { u.add2(8u16) }
fn main() { println!("{}", go(1u16)); }
