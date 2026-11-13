// expect: L0306
// Weak::upgrade shares the RefCell.
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let x = 5;
    let r = Rc::new(RefCell::new(&x));
    let w = Rc::downgrade(&r);
    {
        let y = 6;
        if let Some(s) = w.upgrade() {
            s.replace(&y);
        }
    }
    println!("{}", r.borrow());
}
