// expect: L0733
// ARCH 2.8, rustc E0521: `impl dyn Shape` is `impl dyn Shape + 'static`; a `&dyn Shape` param is not.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
fn f(d: &dyn Shape) -> u32 { d.double() }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
fn main() { println!("{}", f(&Sq(2))); }
