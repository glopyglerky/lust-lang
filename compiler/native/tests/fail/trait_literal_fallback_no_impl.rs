// expect: L0612
// Two impls match the literal, so it waits for the fallback; i32 then
// has no impl (rustc E0277).
trait Tr { fn m(&self) -> u32; }
impl Tr for u8 { fn m(&self) -> u32 { 8 } }
impl Tr for u16 { fn m(&self) -> u32 { 16 } }
fn g<T: Tr>(t: T) -> u32 { t.m() }
fn main() { println!("{}", g(5)); }
