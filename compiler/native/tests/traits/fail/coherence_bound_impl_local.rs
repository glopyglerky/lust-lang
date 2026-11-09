// rustc-error: E0119
// lust-expect: L0615
trait M {}
struct S;
trait Tr { fn v(&self) -> u8; }
impl<T: M> Tr for T { fn v(&self) -> u8 { 1 } }
impl Tr for S { fn v(&self) -> u8 { 2 } }
impl M for S {}
fn main() { println!("{}", S.v()); }
