// rustc-error: E0643
// lust-expect: L0110
trait Tr { fn f<A: Copy>(&self, a: A) -> u8; }
struct S;
impl Tr for S { fn f(&self, a: impl Copy) -> u8 { 1 } }
fn main() { println!("x"); }
