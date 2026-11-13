// panic: RefCell already borrowed
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(1);
    let a = c.borrow();
    println!("got {}", a);
    let b = c.borrow_mut();
    println!("never {}", b);
}
