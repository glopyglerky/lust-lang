// rustc-error: E0049
// lust-expect: L0110
trait Tr { fn f<A>(&self, a: A) -> u8; }
struct S;
impl Tr for S { fn f<A, B>(&self, a: A) -> u8 { 1 } }
fn main() { println!("x"); }
