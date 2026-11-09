// `RefCell::clone` borrows, and its panic reports this call site.
use std::cell::RefCell;

fn main() {
    let c = RefCell::new(1);
    let _a = c.borrow_mut();
    let x = c.clone();
    println!("{x:?}");
}
