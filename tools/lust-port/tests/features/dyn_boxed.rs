// feature: dyn.boxed
// form: Box<dyn Trait> owned value
trait Shape { fn area(&self) -> i32; }
struct Sq { side: i32 }
impl Shape for Sq { fn area(&self) -> i32 { self.side * self.side } }
fn main() {
    let b: Box<dyn Shape> = Box::new(Sq { side: 4 });
    println!("{}", b.area());
}
