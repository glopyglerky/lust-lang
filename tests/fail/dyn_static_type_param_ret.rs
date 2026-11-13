// expect: L0705
// ARCH 2.8, rustc E0310: `T` may hold a borrow, so it cannot become `dyn Shape + 'static`.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
fn g<T: Shape>(t: &T) -> &(dyn Shape + 'static) { t }
fn main() { println!("{}", g(&Sq(1)).area()); }
