// A generic body whose method call reaches Self through `Box` or `Rc`
// (`b.read()` with `b: &Box<&X>`): each instance replays the template's
// pick through the same autoderef steps, for a concrete and a dyn `X`.
use std::rc::Rc;
trait Read {
    fn read(&self) -> u32;
}
struct Number {
    n: u32,
}
impl Read for Number {
    fn read(&self) -> u32 {
        self.n
    }
}
fn get_box<X: Read + ?Sized>(b: &Box<&X>) -> u32 {
    b.read()
}
fn get_rc<X: Read + ?Sized>(b: &Rc<&X>) -> u32 {
    b.read() + 1
}
fn main() {
    let n = Number { n: 7 };
    let d: &dyn Read = &n;
    println!("{} {}", get_box(&Box::new(&n)), get_box(&Box::new(d)));
    println!("{} {}", get_rc(&Rc::new(&n)), get_rc(&Rc::new(d)));
}
