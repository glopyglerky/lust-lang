// expect: L0615
// rustc E0119: `dyn Sub2` implements the supertrait `Other`, so the
// blanket impl already covers it.
trait Other { fn o(&self) -> i32; }
trait Sub2: Other { fn s(&self) -> i32; }
trait Tr { fn m(&self) -> i32; }
impl<T: Other + ?Sized> Tr for T { fn m(&self) -> i32 { 1 } }
impl Tr for dyn Sub2 { fn m(&self) -> i32 { 2 } }
fn main() {}
