trait Base { fn b(&self) -> u8; }
trait Sub: Base { fn s(&self) -> u8; }
struct S;
impl Base for S { fn b(&self) -> u8 { 1 } }
impl Sub for S { fn s(&self) -> u8 { self.b() + 1 } }
fn main() { println!("{}", S.s()); }
