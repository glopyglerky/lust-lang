// expect: L0311
use std::rc::Rc;
use std::cell::RefCell;
fn main() {
    let a = Rc::new(RefCell::new(1));
    let mut m = a.borrow_mut();
    *m += 1;
    drop(a);
    println!("{}", m);
}
