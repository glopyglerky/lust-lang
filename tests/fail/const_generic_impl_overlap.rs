// expect: L0615
// rustc E0119: [T; N] covers [u8; 3].
trait Tr { fn t(&self) -> u8; }
impl<T, const N: usize> Tr for [T; N] { fn t(&self) -> u8 { 1 } }
impl Tr for [u8; 3] { fn t(&self) -> u8 { 2 } }
fn main() { println!("{}", [1u8, 2, 3].t()); }
