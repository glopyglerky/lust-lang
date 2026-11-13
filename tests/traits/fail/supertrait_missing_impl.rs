// rustc-error: E0277
// lust-expect: L0612
trait Base { fn b(&self) -> u8; }
trait Sub: Base { fn s(&self) -> u8; }
struct S;
impl Sub for S { fn s(&self) -> u8 { 1 } }
fn main() { println!("{}", S.s()); }
