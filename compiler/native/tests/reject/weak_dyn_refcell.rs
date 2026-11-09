// expect: L0641
// Valid Rust: a Weak of `RefCell<dyn Tr>` is not built yet.
use std::cell::RefCell;
use std::rc::Rc;
trait Tr {
    fn v(&self) -> u32;
}
struct A;
impl Tr for A {
    fn v(&self) -> u32 {
        1
    }
}
fn main() {
    let r: Rc<RefCell<dyn Tr>> = Rc::new(RefCell::new(A));
    let w = Rc::downgrade(&r);
    println!("{}", w.upgrade().map_or(0, |x| x.borrow().v()));
}
