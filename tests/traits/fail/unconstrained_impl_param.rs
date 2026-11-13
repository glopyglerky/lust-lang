// rustc-error: E0207
// lust-expect: L0110
trait Tr { fn v(&self) -> u8; }
struct S;
impl<T> Tr for S { fn v(&self) -> u8 { 1 } }
fn main() { println!("{}", S.v()); }
