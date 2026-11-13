// expect: L0733
// rustc: lifetime may not live long enough. The annotation `Rc<dyn P>` equals the expected
// input `Rc<dyn P + 'a>`, which is not `'static`.
use std::rc::Rc;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
fn need(p: &Rc<dyn P>) -> u32 { p.id() }
fn each<'a>(k: &mut dyn FnMut(Rc<dyn P + 'a>) -> u32, x: Rc<dyn P + 'a>) -> u32 { k(x) }
fn g<'a>(x: Rc<dyn P + 'a>) -> u32 { each(&mut |p: Rc<dyn P>| need(&p), x) }
fn main() { println!("{}", g(Rc::new(A(1)))); }
