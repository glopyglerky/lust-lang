// expect: L0311
// A guard keeps its RefCell borrowed until its drop.
use std::cell::RefCell;
fn main() {
    let c = RefCell::new(1);
    let _r = c.borrow();
    let v = c.into_inner();
    println!("{}", v);
}
