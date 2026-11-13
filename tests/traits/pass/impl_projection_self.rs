trait X { type A; }
struct S;
impl X for S { type A = u8; }
trait Tr { fn v(&self) -> u8; }
impl Tr for <S as X>::A { fn v(&self) -> u8 { 9 } }
fn main() { println!("{}", 5u8.v()); }
