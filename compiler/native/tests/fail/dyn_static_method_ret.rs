// expect: L0306
// ARCH 2.8, rustc E0597: `get` returns a `&dyn` that lives only as long as `g`.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
trait Get { fn get(&self) -> &dyn Shape; }
struct G(Sq);
impl Get for G { fn get(&self) -> &dyn Shape { &self.0 } }
fn main() { let g = G(Sq(2)); println!("{}", g.get().double()); }
