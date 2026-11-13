// expect: L0308
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(vec![1]);
    let r = c.borrow();
    r.push(2);
}
