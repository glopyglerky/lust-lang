use std::cell::*;
use std::rc::*;
fn main() {
    let c: Cell<u8> = Cell::new(1);
    let r = Rc::new(RefCell::new(2));
    let w: Weak<RefCell<i32>> = Rc::downgrade(&r);
    println!("{} {} {}", c.get(), r.borrow(), w.strong_count());
}
