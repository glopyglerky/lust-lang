// expect: L0306
// An Rc clone shares the RefCell, both ways.
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let x = 5;
    let r = Rc::new(RefCell::new(vec![&x]));
    let r2 = Rc::clone(&r);
    {
        let y = 6;
        r2.borrow_mut().push(&y);
    }
    println!("{}", r.borrow().len());
}
