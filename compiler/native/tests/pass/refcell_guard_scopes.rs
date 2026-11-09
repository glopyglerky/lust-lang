// RefCell guards drop at the end of their statement or scope, as
// Rust temporaries do, so these borrows never overlap.
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let c = Rc::new(RefCell::new(5));
    let n = *c.borrow() + 1;
    *c.borrow_mut() = n;
    let v = c.borrow().clone();
    if *c.borrow() > 3 {
        *c.borrow_mut() += 1;
    }
    match *c.borrow() {
        7 => println!("seven"),
        x => println!("{}", x),
    }
    let r = c.borrow();
    println!("{}", *r);
}
