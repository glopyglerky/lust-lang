// expect: L0306
// ARCH 2.8, rustc E0597: a dyn made from `&Sq` lives only as long as `s`.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
impl Shape for &Sq { fn area(&self) -> u32 { self.0 + 1 } }
fn main() { let s = Sq(1); let r = &s; let d: &dyn Shape = &r; println!("{}", d.double()); }
