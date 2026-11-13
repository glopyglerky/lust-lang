// rustc-error: E0276
// lust-expect: L0110
trait T { fn a<U>(&self, u: U) -> u8; }
struct X;
impl T for X { fn a<U: Copy>(&self, u: U) -> u8 { 1 } }
fn main() { println!("{}", X.a(5u8)); }
