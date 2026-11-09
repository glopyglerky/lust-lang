// expect: L0733
// rustc: lifetime may not live long enough. `x`'s object lifetime is `'a`, and `need` takes
// `&Rc<dyn P>` (`'static`); the body also makes a dyn value from a borrow.
use std::rc::Rc;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn need(p: &Rc<dyn P>) -> u32 { p.id() }
fn f<'a>(x: Rc<dyn P + 'a>) -> u32 {
    let mut n = 0u32;
    let k: &mut dyn FnMut() = &mut || n += 1;
    k();
    need(&x) + n
}
fn main() { println!("{}", f(Rc::new(A(1)))); }
