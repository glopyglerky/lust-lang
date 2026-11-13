// ts-rust ls/crossproject.rs: a closure parameter annotated `Rc<dyn P>` passed as
// `&mut dyn FnMut(Rc<dyn P>)` takes the expected `'static` object lifetime (rustc equates the
// annotation with the expected input), so it may go to a `'static` use, even in a body that also
// makes a non-`'static` dyn value from a borrow.
use std::rc::Rc;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
struct Item { p: Rc<dyn P> }
fn can(p: &Rc<dyn P>) -> bool { p.id() > 0 }
fn each(f: &mut dyn FnMut(Rc<dyn P>) -> bool) { for i in 0..4 { if !f(Rc::new(A(i))) { break; } } }
fn main() {
    let mut items = Vec::new();
    let mut seen = 0;
    each(&mut |p: Rc<dyn P>| {
        seen += 1;
        if !can(&p) { return true; }
        items.push(Item { p });
        true
    });
    println!("{} {}", seen, items.iter().map(|i| i.p.id()).sum::<u32>());
}
