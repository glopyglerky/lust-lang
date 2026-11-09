// expect: L0311
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(1);
    let r = c.borrow();
    let v = c.into_inner();
    println!("{} {}", r, v);
}
