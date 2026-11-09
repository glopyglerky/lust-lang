// panic: RefCell already mutably borrowed
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(vec![1]);
    let m = c.borrow_mut();
    println!("len {}", c.borrow().len());
    drop(m);
}
