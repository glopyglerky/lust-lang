// expect: L0612
// `Weak<S>` to `Weak<dyn Tr>` needs `S: Tr`.
use std::rc::{Rc, Weak};
trait Tr {
    fn v(&self) -> u32;
}
struct S;
fn main() {
    let s = Rc::new(S);
    let w: Weak<S> = Rc::downgrade(&s);
    let d: Weak<dyn Tr> = w;
    println!("{}", d.upgrade().is_some());
}
