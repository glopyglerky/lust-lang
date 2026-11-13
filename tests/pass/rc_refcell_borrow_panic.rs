// panic: RefCell already borrowed
// Rc<RefCell<T>> fixture set: a second mutable borrow through a clone panics.
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let a = Rc::new(RefCell::new(vec![1]));
    let b = a.clone();
    let r = a.borrow();
    println!("{} {}", r.len(), b.try_borrow_mut().is_err());
    let _w = b.borrow_mut();
    println!("never");
}
