// expect: L0612
// &RefCell<T> is not UnwindSafe: RefCell is not RefUnwindSafe (rustc E0277).
use std::panic;
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(1);
    let r = panic::catch_unwind(|| { *c.borrow_mut() += 1; });
    println!("{:?}", r.is_ok());
}
