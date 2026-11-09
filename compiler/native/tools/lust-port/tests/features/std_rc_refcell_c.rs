// feature: std.rc_refcell
// form: Rc<RefCell<T>> shared and mutated through two handles, the ts-rust core pattern
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
    let shared: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(vec![1]));
    let again: Rc<RefCell<Vec<i32>>> = Rc::clone(&shared);
    again.borrow_mut().push(2);
    println!("{} {}", shared.borrow().len(), Rc::strong_count(&shared));
}
