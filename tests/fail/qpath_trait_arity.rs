// expect: L0110
// rustc E0107: the trait of a qualified path takes its declared number of arguments.
trait B<X> { fn b(&self) -> u8; }
struct A;
impl B<u8> for A { fn b(&self) -> u8 { 1 } }
fn main() { println!("{}", <A as B<u8, u16>>::b(&A)); }
