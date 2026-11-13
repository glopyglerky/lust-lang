// expect: L0612
// A float literal never selects an integer impl (rustc E0277).
trait Tr { fn m(&self) -> u32; }
impl Tr for u8 { fn m(&self) -> u32 { 8 } }
fn g<T: Tr>(t: T) -> u32 { t.m() }
fn main() { println!("{}", g(2.5)); }
