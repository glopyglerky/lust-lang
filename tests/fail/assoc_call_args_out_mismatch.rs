// expect: L0110
// rustc E0308: Conv<u8>::Out is u16.
trait Conv<T> { type Out; fn conv(t: T) -> Self::Out; }
struct S;
impl Conv<u8> for S { type Out = u16; fn conv(t: u8) -> u16 { t as u16 * 2 } }
impl Conv<u16> for S { type Out = u32; fn conv(t: u16) -> u32 { t as u32 * 3 } }
fn main() { let a: u32 = S::conv(5u8); println!("{}", a); }
