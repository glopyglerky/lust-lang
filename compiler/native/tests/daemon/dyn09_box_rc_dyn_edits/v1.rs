// Owning dyn edits on one warm daemon: `Box<dyn Tr>` and `Rc<dyn Tr>`
// fields, a Drop impl added to a concrete type (the vtable drop word),
// its layout (the size word), and a field switched from Box to Rc.
use std::rc::Rc;
trait Shape {
    fn area(&self) -> u32;
}
struct Sq {
    side: u32,
}
struct Tag(&'static str);
impl Shape for Sq {
    fn area(&self) -> u32 {
        self.side * self.side
    }
}
impl Shape for Tag {
    fn area(&self) -> u32 {
        self.0.len() as u32
    }
}
struct Scene {
    main: Box<dyn Shape>,
    shared: Rc<dyn Shape>,
}
fn main() {
    let shared: Rc<dyn Shape> = Rc::new(Tag("shared"));
    let a = Scene { main: Box::new(Sq { side: 3 }), shared: shared.clone() };
    let b = Scene { main: Box::new(Tag("bb")), shared };
    println!("{} {}", a.main.area() + a.shared.area(), b.main.area());
    println!("{}", Rc::strong_count(&b.shared));
    drop(a);
    println!("{}", Rc::strong_count(&b.shared));
}
