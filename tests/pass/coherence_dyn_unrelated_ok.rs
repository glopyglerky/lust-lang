// `dyn Unrel` does not implement `Other`, so the blanket impl does
// not cover it and the two impls do not overlap.
trait Other { fn o(&self) -> i32; }
trait Unrel { fn u(&self) -> i32; }
trait Tr { fn m(&self) -> i32; }
impl<T: Other + ?Sized> Tr for T { fn m(&self) -> i32 { 1 } }
impl Tr for dyn Unrel { fn m(&self) -> i32 { 2 } }
fn main() { println!("ok"); }
