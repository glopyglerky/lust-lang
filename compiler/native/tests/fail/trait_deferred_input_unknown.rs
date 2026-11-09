// expect: L0610
// Nothing fixes the trait input, so no impl can be picked (rustc E0283).
trait Conv<T> { fn conv(&self) -> T; }
struct S(u8);
impl Conv<u16> for S { fn conv(&self) -> u16 { 1 } }
impl Conv<u32> for S { fn conv(&self) -> u32 { 2 } }
fn main() { let s = S(5); let _a = s.conv(); }
