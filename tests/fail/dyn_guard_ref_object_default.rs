// expect: L0733
// rustc: lifetime may not live long enough. `Ref<'b, dyn P>` is `Ref<'b, dyn P + 'b>` (the
// guard's `T: ?Sized + 'b` bound gives the object lifetime), not `'static`.
use std::cell::{Ref, RefCell};
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn f<'b>(r: Ref<'b, dyn P>) -> u32 {
    let x: &(dyn P + 'static) = &*r;
    x.id()
}
fn main() {
    let c: Box<RefCell<dyn P>> = Box::new(RefCell::new(A(3)));
    println!("{}", f(c.borrow()));
}
