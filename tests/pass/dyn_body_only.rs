// ARCH 2.8: dyn types that only bodies name (no signature has one) still get vtables and instances.
trait Shape { fn area(&self) -> u32; fn name(&self) -> String { String::from("shape") } }
struct Sq(u32);
struct Plus<T>(T);
impl Shape for Sq { fn area(&self) -> u32 { self.0 * self.0 } fn name(&self) -> String { String::from("sq") } }
impl<T: Shape> Shape for Plus<T> { fn area(&self) -> u32 { self.0.area() + 1 } }
fn main() {
    let s = Sq(3);
    let p = Plus(Sq(2));
    let a: &dyn Shape = &s;
    let b = &p as &dyn Shape;
    let all: Vec<&dyn Shape> = vec![a, b];
    let maybe: Option<&dyn Shape> = Some(&p);
    for x in &all { println!("{} {}", x.name(), x.area()); }
    if let Some(m) = maybe { println!("{} {}", m.name(), m.area()); }
}
