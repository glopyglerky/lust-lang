// `Box<dyn Tr>` values: a Vec of boxes, default and overriding methods,
// `&*b` to `&dyn Tr`, and the fat pointer size.
trait Shape { fn area(&self) -> i32; fn name(&self) -> String { String::from("shape") } }
struct Sq { side: i32 }
struct Circle { r: i32, label: String }
impl Shape for Sq { fn area(&self) -> i32 { self.side * self.side } }
impl Shape for Circle { fn area(&self) -> i32 { 3 * self.r * self.r } fn name(&self) -> String { self.label.clone() } }
fn total(v: &[Box<dyn Shape>]) -> i32 { v.iter().map(|s| s.area()).sum() }
fn main() {
    let b: Box<dyn Shape> = Box::new(Sq { side: 4 });
    println!("{} {}", b.area(), b.name());
    let v: Vec<Box<dyn Shape>> = vec![Box::new(Sq { side: 2 }), Box::new(Circle { r: 1, label: String::from("c") })];
    println!("{}", total(&v));
    for s in &v { println!("{} {}", s.name(), s.area()); }
    let r: &dyn Shape = &*b;
    println!("{}", r.area());
    println!("{}", std::mem::size_of::<Box<dyn Shape>>());
}
