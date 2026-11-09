// panic: RefCell already mutably borrowed
// with_borrow inside with_borrow_mut of the same key panics with Rust's
// RefCell message.
use std::cell::RefCell;

thread_local! {
    static LOG: RefCell<Vec<u32>> = RefCell::new(Vec::new());
}

fn main() {
    LOG.with_borrow_mut(|v| {
        v.push(1);
        println!("pushed");
        LOG.with_borrow(|w| println!("never {}", w.len()));
    });
}
