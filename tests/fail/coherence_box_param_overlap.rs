// expect: L0615
// rustc E0119: Box<U> is Clone for any U: Clone; nothing disproves it.
trait Tr {}
impl<T: Clone> Tr for T {}
impl<U> Tr for Box<U> {}
fn main() {}
