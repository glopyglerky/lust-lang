// panic: RefCell already mutably borrowed
// `.map(RefCell::borrow)` through Option::map's FnOnce bound while the
// cell is mutably borrowed: the borrow panics at core's FnOnce shim
// (core/src/ops/function.rs:250:5), as rustc's fn item shim reports.
use std::cell::RefCell;

fn main() {
    let c = RefCell::new(vec![1u8]);
    let o = Some(&c);
    println!("{:?}", o.map(RefCell::borrow).map(|g| g.len()));
    let _m = c.borrow_mut();
    let n = o.map(RefCell::borrow).map(|g| g.len());
    println!("{:?}", n);
}
