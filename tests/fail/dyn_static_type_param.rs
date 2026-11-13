// expect: L0733
// ARCH 2.8, rustc E0310: `T` may hold a borrow, so `dyn Shape` from it is not `'static`.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
fn g<T: Shape>(t: &T) -> u32 { let d: &dyn Shape = t; d.double() }
fn main() { println!("{}", g(&Sq(1))); }
