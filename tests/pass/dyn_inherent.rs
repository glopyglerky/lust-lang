// ARCH 2.8: `impl dyn Trait` methods call through the vtable.
trait Shape { fn area(&self) -> u32; }
impl dyn Shape { fn double(&self) -> u32 { self.area() * 2 } fn tag() -> u8 { 9 } }
struct Sq(u32);
impl Shape for Sq { fn area(&self) -> u32 { self.0 * self.0 } }
fn main() {
    let s = Sq(3);
    let d: &dyn Shape = &s;
    println!("{} {}", d.area(), d.double());
}
