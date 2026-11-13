// expect: L0615
// rustc E0119: Box<u8> is Clone, so the blanket over T: Clone overlaps.
trait Tr {}
impl<T: Clone> Tr for T {}
impl Tr for Box<u8> {}
fn main() {}
