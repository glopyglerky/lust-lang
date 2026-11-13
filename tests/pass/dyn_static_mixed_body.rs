// ARCH 2.8: rustc infers a `'static` object lifetime for `d`, next to the
// `&dyn Shape` param `p` in the same body; the borrow checker tells them apart.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 } }
fn f(p: &dyn Shape, s: &Sq) -> u32 { let d: &dyn Shape = s; d.double() + p.area() }
fn main() { println!("{}", f(&Sq(1), &Sq(3))); }
