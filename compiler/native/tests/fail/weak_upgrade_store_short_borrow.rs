// expect: L0306
// Weak::upgrade shares the RefCell's T with the Rc.
use std::cell::RefCell;
use std::rc::{Rc, Weak};
fn main() {
    let x = 5;
    let r = Rc::new(RefCell::new(&x));
    let w: Weak<RefCell<&i32>> = Rc::downgrade(&r);
    {
        let y = 6;
        if let Some(s) = w.upgrade() {
            *s.borrow_mut() = &y;
        }
    }
    println!("{}", r.borrow());
}
