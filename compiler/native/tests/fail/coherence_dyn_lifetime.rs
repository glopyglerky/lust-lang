// expect: L0615
// rustc E0119: dyn types that differ only in the object lifetime
// bound are the same type for overlap.
trait Other { fn o(&self) -> i32; }
trait Tr { fn m(&self) -> i32; }
impl Tr for dyn Other + 'static { fn m(&self) -> i32 { 1 } }
impl<'a> Tr for dyn Other + 'a { fn m(&self) -> i32 { 2 } }
fn main() {}
