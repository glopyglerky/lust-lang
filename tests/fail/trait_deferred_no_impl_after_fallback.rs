// expect: L0612
// The trait method is picked, its impl waits for the literal; the
// fallback makes it i32, which has no impl (rustc E0277).
trait Tr { fn m(self) -> u32; }
impl Tr for u8 { fn m(self) -> u32 { 8 } }
impl Tr for u16 { fn m(self) -> u32 { 16 } }
fn main() { let x = 5; println!("{}", x.m()); }
