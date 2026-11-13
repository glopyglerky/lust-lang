// expect: L0102
// rustc E0405: the trait of a qualified path must exist.
trait B { fn b(&self) -> u8; }
struct A;
impl B for A { fn b(&self) -> u8 { 1 } }
fn main() { println!("{}", <A as Nope>::b(&A)); }
