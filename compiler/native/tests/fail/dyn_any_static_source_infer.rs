// expect: L0733
// rustc E0521: borrowed data escapes. The coercion's source type holds an inference variable
// that a later store binds to `&u8`, so the `'static` dyn value would hold a borrow.
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
fn mk(v: &u8) -> Rc<dyn Any> {
    let c = Rc::new(RefCell::new(None));
    let a: Rc<dyn Any> = c.clone();
    *c.borrow_mut() = Some(v);
    a
}
fn main() { let x = 5u8; println!("{}", mk(&x).is::<u8>()); }
