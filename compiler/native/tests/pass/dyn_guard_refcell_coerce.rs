// A `RefCell` guard over `dyn P` takes the guard's lifetime as its object lifetime
// (`RefMut<'b, dyn P + 'b>`), and a guard over a `'static` object coerces to it at a
// return and at an argument whose param type is written. A push and a reassignment keep
// it. `Ref` is covariant, so an `Option<Ref<..>>` shortens too.
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
trait P { fn id(&self) -> u32; }
struct A(u32);
impl P for A { fn id(&self) -> u32 { self.0 } }
struct Printer {
    w: Option<Rc<RefCell<dyn P>>>,
}
impl Printer {
    fn writer(&self) -> RefMut<'_, dyn P> {
        self.w.as_ref().expect("nil writer").borrow_mut()
    }
    fn reader(&self) -> Ref<'_, dyn P> {
        self.w.as_ref().unwrap().borrow()
    }
}
fn shrink<'b>(r: RefMut<'b, dyn P + 'static>) -> RefMut<'b, dyn P> {
    r
}
fn shrink_opt<'b>(r: Option<Ref<'b, dyn P + 'static>>) -> Option<Ref<'b, dyn P>> {
    r
}
fn take<'b>(r: RefMut<'b, dyn P>) -> u32 {
    r.id()
}
fn push<'b>(c: &'b Rc<RefCell<dyn P>>, v: &mut Vec<RefMut<'b, dyn P>>) {
    v.push(c.borrow_mut());
}
fn use_static<'b>(r: Ref<'b, dyn P + 'static>) -> u32 {
    let x: &(dyn P + 'static) = &*r;
    x.id()
}
fn main() {
    let p = Printer { w: Some(Rc::new(RefCell::new(A(4)))) };
    println!("{} {}", p.writer().id(), p.reader().id());
    let c: Rc<RefCell<dyn P>> = Rc::new(RefCell::new(A(7)));
    println!("{}", shrink(c.borrow_mut()).id());
    println!("{}", shrink_opt(Some(c.borrow())).map_or(0, |r| r.id()));
    let d = c.clone();
    let mut v = Vec::new();
    push(&d, &mut v);
    println!("{} {}", v.len(), v[0].id());
    drop(v);
    println!("{} {}", take(c.borrow_mut()), use_static(c.borrow()));
    let e: Rc<RefCell<dyn P>> = Rc::new(RefCell::new(A(9)));
    let mut g: RefMut<dyn P> = c.borrow_mut();
    println!("{}", g.id());
    drop(g);
    g = e.borrow_mut();
    println!("{}", g.id());
}
