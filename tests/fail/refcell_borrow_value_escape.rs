// expect: L0306
// rustc E0597: the guard from `.map(RefCell::borrow)` borrows the cell,
// which ends before the guard is used.
use std::cell::RefCell;
fn main() {
    let r = {
        let c = RefCell::new(1);
        Some(&c).map(RefCell::borrow)
    };
    println!("{:?}", r.is_some());
}
