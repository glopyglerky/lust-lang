// expect: L0631
// `Box::new` is not const, even under a const constructor (E0015).
use std::cell::RefCell;
thread_local! {
    static S: RefCell<Option<Box<i32>>> = const { RefCell::new(Some(Box::new(1))) };
}
fn main() {
    S.with_borrow(|v| println!("{:?}", v));
}
