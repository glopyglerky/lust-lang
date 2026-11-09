// expect: L0733
// ARCH 2.8, rustc E0521: `self` in `impl dyn Shape + '_` is not `'static`.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
impl dyn Shape + '_ { fn h(&self) -> u32 { self.double() } }
fn main() { let s = Sq(1); let d: &dyn Shape = &s; println!("{}", d.h()); }
